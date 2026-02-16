use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::crypto::CurveKeyPair;
use crate::serde_bigint::{deserialize_bigint, serialize_bigint, SerdeBigInt};

pub fn load_default_keys() -> Result<Keys> {
    let mut v = serde_json::from_str(std::include_str!("../keys.json"))?;
    preprocess_object(&mut v).context("unable to preprocess object")?;
    Ok(serde_json::from_value(v)?)
}

pub fn load_keys<P: AsRef<Path>>(path: P) -> Result<Keys> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut v = serde_json::from_reader(reader)?;
    preprocess_object(&mut v).context("unable to preprocess object")?;
    Ok(serde_json::from_value(v)?)
}

fn preprocess_object(j: &mut Value) -> Option<()> {
    let object = j.as_object_mut()?;
    let binkeys = object.get_mut("BINK")?.as_object_mut()?;

    for (_, bink) in binkeys {
        let bink_obj = bink.as_object_mut()?;

        if let (Some(p), Some(a), Some(b)) = (bink_obj.remove("p"), bink_obj.remove("a"), bink_obj.remove("b")) {
            bink_obj.insert("curve".into(), json!({
                "p": p,
                "a": a,
                "b": b,
            }));
        }

        if let (Some(g), Some(public)) = (bink_obj.remove("g"), bink_obj.remove("pub")) {
            bink_obj.insert("public".into(), json!({
                "g": g,
                "k": public,
            }));
        }

        if let (Some(n), Some(private)) = (bink_obj.remove("n"), bink_obj.remove("priv")) {
            bink_obj.insert("private".into(), json!({
                "n": n,
                "key": private,
            }));
        }
    }

    Some(())
}

pub enum ProductOrFlavourRef<'a> {
    Product {
        product: &'a Product,
    },
    ProductFlavour {
        product: &'a Product,
        flavour: &'a ProductFlavour,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Keys {
    pub products: HashMap<String, Product>,
    #[serde(rename = "BINK")]
    pub bink: HashMap<String, CurveKeyPair>,
    pub activation: HashMap<String, Activation>,
}

impl<'a> Keys {
    pub fn get_products_by_bink_id(&'a self, id: u32) -> Vec<ProductOrFlavourRef<'a>> {
        let bink_id_str = format!("{:02X}", id);
        let mut result = Vec::new();

        for (_, product) in &self.products {
            let product_matches = if let Some(product_binkeys) = &product.bink {
                let mut bink_id_matches = false;
                for product_bink_id_str in product_binkeys {
                    if product_bink_id_str.eq_ignore_ascii_case(&bink_id_str) {
                        bink_id_matches = true;
                        break;
                    }
                }
                bink_id_matches
            } else {
                false
            };

            let mut any_flavour_matches = false;
            if let Some(flavours) = &product.flavours {
                for (_, flavour) in flavours {
                    let flavour_matches = if let Some(flavour_binkeys) = &flavour.bink {
                        let mut bink_id_matches = false;
                        for flavour_bink_id_str in flavour_binkeys {
                            if flavour_bink_id_str.eq_ignore_ascii_case(&bink_id_str) {
                                bink_id_matches = true;
                                break;
                            }
                        }
                        bink_id_matches
                    } else {
                        false
                    };

                    if flavour_matches {
                        result.push(ProductOrFlavourRef::ProductFlavour { product, flavour });
                        any_flavour_matches = true;
                    }
                }
            }

            if !any_flavour_matches && product_matches {
                result.push(ProductOrFlavourRef::Product { product });
            }
        }

        result
    }

    pub fn get_bink_by_id(&self, id: u32) -> Option<&CurveKeyPair> {
        let bink_id_str = format!("{:02X}", id);
        self.bink.get(&bink_id_str)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Activation {
    pub name: String,
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub p: BigInt,
    pub x: IndexMap<u32, SerdeBigInt>,
    #[serde(
        rename = "priv",
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub private: BigInt,
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub iid_key: BigInt,
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub non_residue: BigInt,
    #[serde(
        serialize_with = "serialize_bigint",
        deserialize_with = "deserialize_bigint"
    )]
    pub quotient: BigInt,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivationMetadata {
    pub flavour: String,
    pub version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    #[serde(rename = "type")]
    pub activation_type: String,
    pub tags: Option<Vec<String>>,
    pub activation: Option<ActivationMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dpc {
    pub min: u32,
    pub max: u32,
    pub is_evaluation: bool,
    pub activation_grace_days: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Product {
    pub name: String,
    #[serde(rename = "BINK")]
    pub bink: Option<Vec<String>>,
    pub meta: Option<Metadata>,
    pub flavours: Option<HashMap<String, ProductFlavour>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProductFlavour {
    pub name: String,
    #[serde(rename = "BINK")]
    pub bink: Option<Vec<String>>,
    pub meta: Option<Metadata>,
    #[serde(rename = "DPC")]
    pub dpc: Option<HashMap<String, Vec<Dpc>>>,
    pub exclusions: Option<Vec<String>>,
}
