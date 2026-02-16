#pragma author Alexander Sagen <alexander@sagen.me>
#pragma description BINK
#pragma MIME application/vnd.ms-publisher

struct BINKHDR {
    u32 size;
	u32 field_count;
	u32 checksum;
	u32 version;
    u32 key_size;
    u32 hash_len;
    u32 sig_len;
    if (field_count == 9) {
        u32 auth_len;
        u32 pid_len;
    }
};

struct BINK {
	u32 id;
	BINKHDR hdr;
	u8 p[hdr.key_size * 4];
	u8 a[hdr.key_size * 4];
	u8 b[hdr.key_size * 4];
	u8 gx[hdr.key_size * 4];
	u8 gy[hdr.key_size * 4];
	u8 kx[hdr.key_size * 4];
	u8 ky[hdr.key_size * 4];
};

BINK bink @ 0x00;