#![allow(non_snake_case)]

use std::mem::ManuallyDrop;

use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize, IDispatch, IDispatch_Impl, IDispatch_Vtbl,
};
use windows::core::*;

const LICDLL_CLSID: GUID = GUID::from_values(0xACADF079, 0xCBCD, 0x4032, [0x83, 0xF2, 0xFA, 0x47, 0xC4, 0xDB, 0x09, 0x6F]);

#[derive(Debug, Clone)]
pub struct LicenseAgent(ManuallyDrop<ICOMLicenseAgent>);

impl LicenseAgent {
    pub fn new() -> Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() }?;
        let inner = unsafe { CoCreateInstance(&LICDLL_CLSID, None, CLSCTX_INPROC_SERVER) }?;
        let inner = ManuallyDrop::new(inner);
		Self(inner).initialize()
    }

    fn initialize(self) -> Result<Self> {
		let mut ret_code = ERROR_SUCCESS;
		unsafe { self.0.Initialize(0xC475, 3, BSTR::new(), &mut ret_code) }?;

		if ret_code == ERROR_SUCCESS {
			Ok(self)
		} else {
			Err(ret_code.to_hresult().into())
		}
    }

    pub fn get_expiration_info(&mut self) -> Result<ExpirationInfo> {
		let mut expiration_info = ExpirationInfo { wpa_left: 0, eval_left: 0 };
        unsafe { (self.0.vtable().GetExpirationInfo)(self.0.as_raw(), &mut expiration_info.wpa_left, &mut expiration_info.eval_left).ok() }?;
		Ok(expiration_info)
    }

    pub fn generate_installation_id(&mut self) -> Result<String> {
		let mut installation_id = BSTR::new();
		unsafe { (self.0.vtable().GenerateInstallationId)(self.0.as_raw(), &mut installation_id).ok() }?;
		Ok(installation_id.to_string())
    }

    pub fn deposit_confirmation_id(&mut self, val: &str) -> Result<()> {
		let wide_chars: Vec<u16> = val.encode_utf16().collect();
		let wstr = BSTR::from_wide(&wide_chars);
		let mut ret_code = ERROR_SUCCESS;
		unsafe { (self.0.vtable().DepositConfirmationId)(self.0.as_raw(), wstr, &mut ret_code).ok() }?;
		ret_code.ok()
    }

    pub fn get_product_id(&mut self) -> Result<String> {
		let mut product_id = BSTR::new();
		unsafe { (self.0.vtable().GetProductID)(self.0.as_raw(), &mut product_id).ok() }?;
		Ok(product_id.to_string())
    }

    pub fn set_product_key(&mut self, val: &str) -> Result<()> {
		let mut wide_chars: Vec<u16> = val.encode_utf16().chain([0]).collect();
        let mut pwstr = PWSTR::from_raw(wide_chars.as_mut_ptr());
		unsafe { (self.0.vtable().SetProductKey)(self.0.as_raw(), &mut pwstr).ok() }
    }
}

impl Drop for LicenseAgent {
    fn drop(&mut self) {
        unsafe {
            std::mem::drop(ManuallyDrop::take(&mut self.0)); // Calls vtable method Release
            CoUninitialize(); // Must not be called before Release
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpirationInfo {
    pub wpa_left: u32,
    pub eval_left: u32,
}

impl ExpirationInfo {
    pub fn is_activated(&self) -> bool {
        self.wpa_left == 0x7FFFFFFF
    }
}

#[interface("b8cbad79-3f1f-481a-bb0c-e7bbd77bddd1")]
unsafe trait ICOMLicenseAgent: IDispatch {
    fn Initialize(&self, bpc: u32, mode: u32, lic_source: BSTR, ret_code: *mut WIN32_ERROR) -> Result<()>;
    fn GetFirstName(&self, val: *mut BSTR) -> Result<()>;
    fn SetFirstName(&self, new_val: BSTR) -> Result<()>;
    fn GetLastName(&self, val: *mut BSTR) -> Result<()>;
    fn SetLastName(&self, new_val: BSTR) -> Result<()>;
    fn GetOrgName(&self, val: *mut BSTR) -> Result<()>;
    fn SetOrgName(&self, new_val: BSTR) -> Result<()>;
    fn GetEmail(&self, val: *mut BSTR) -> Result<()>;
    fn SetEmail(&self, new_val: BSTR) -> Result<()>;
    fn GetPhone(&self, val: *mut BSTR) -> Result<()>;
    fn SetPhone(&self, new_val: BSTR) -> Result<()>;
    fn GetAddress1(&self, val: *mut BSTR) -> Result<()>;
    fn SetAddress1(&self, new_val: BSTR) -> Result<()>;
    fn GetCity(&self, val: *mut BSTR) -> Result<()>;
    fn SetCity(&self, new_val: BSTR) -> Result<()>;
    fn GetState(&self, val: *mut BSTR) -> Result<()>;
    fn SetState(&self, new_val: BSTR) -> Result<()>;
    fn GetCountryCode(&self, val: *mut BSTR) -> Result<()>;
    fn SetCountryCode(&self, new_val: BSTR) -> Result<()>;
    fn GetCountryDesc(&self, val: *mut BSTR) -> Result<()>;
    fn SetCountryDesc(&self, new_val: BSTR) -> Result<()>;
    fn GetZip(&self, val: *mut BSTR) -> Result<()>;
    fn SetZip(&self, new_val: BSTR) -> Result<()>;
    fn GetIsoLanguage(&self, val: *mut u32) -> Result<()>;
    fn SetIsoLanguage(&self, new_val: u32) -> Result<()>;
    fn GetMSUpdate(&self, val: *mut BSTR) -> Result<()>;
    fn SetMSUpdate(&self, new_val: BSTR) -> Result<()>;
    fn GetMSOffer(&self, val: *mut BSTR) -> Result<()>;
    fn SetMSOffer(&self, new_val: BSTR) -> Result<()>;
    fn GetOtherOffer(&self, val: *mut BSTR) -> Result<()>;
    fn SetOtherOffer(&self, new_val: BSTR) -> Result<()>;
    fn GetAddress2(&self, val: *mut BSTR) -> Result<()>;
    fn SetAddress2(&self, new_val: BSTR) -> Result<()>;
    fn AsyncProcessHandshakeRequest(&self, revise_cust_info: i32) -> Result<()>;
    fn AsyncProcessNewLicenseRequest(&self) -> Result<()>;
    fn AsyncProcessReissueLicenseRequest(&self) -> Result<()>;
    fn AsyncProcessReviseCustInfoRequest(&self) -> Result<()>;
    fn GetAsyncProcessReturnCode(&self, ret_code: *mut WIN32_ERROR) -> Result<()>;
    fn AsyncProcessDroppedLicenseRequest(&self) -> Result<()>;
    fn GenerateInstallationId(&self, val: *mut BSTR) -> Result<()>;
    fn DepositConfirmationId(&self, val: BSTR, ret_code: *mut WIN32_ERROR) -> Result<()>;
    fn GetExpirationInfo(&self, wpa_left: *mut u32, eval_left: *mut u32) -> Result<()>;
    fn AsyncProcessRegistrationRequest(&self) -> Result<()>;
    fn ProcessHandshakeRequest(&self, revise_cust_info: i32) -> Result<()>;
    fn ProcessNewLicenseRequest(&self) -> Result<()>;
    fn ProcessDroppedLicenseRequest(&self) -> Result<()>;
    fn ProcessReissueLicenseRequest(&self) -> Result<()>;
    fn ProcessReviseCustInfoRequest(&self) -> Result<()>;
    fn EnsureInternetConnection(&self) -> Result<()>;
    fn SetProductKey(&self, new_product_key: *mut PWSTR) -> Result<()>;
    fn GetProductID(&self, val: *mut BSTR) -> Result<()>;
    fn VerifyCheckDigits(&self, cid_iid: BSTR, value: *mut i32) -> Result<()>;
}
