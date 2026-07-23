//! # easyofd-signature
//!
//! OFD electronic seal and digital signature operations per GB/T 38540.
//!
//! ## Status: v0.3 — API design complete, cryptographic signing stub

use std::io::{Cursor, Read, Write};

use easyofd_core::OfdResult;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Supported signature algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    /// SM2 with SM3 hash (Chinese national standard)
    Sm2WithSm3,
    /// SHA-256 with RSA
    Sha256WithRsa,
}

/// An OFD electronic seal (stamp).
#[derive(Debug, Clone)]
pub struct ElectronicSeal {
    /// Seal image data (PNG/JPEG).
    pub image_data: Vec<u8>,
    /// Seal name/identifier.
    pub name: String,
    /// Position: (x, y) in mm, page number (1-based).
    pub position: (f64, f64),
    pub page: usize,
}

/// Result of signing an OFD document.
#[derive(Debug)]
pub struct SignedOfd {
    data: Vec<u8>,
}

impl SignedOfd {
    /// Save the signed OFD to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if file I/O fails.
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        std::fs::write(path, self.data).map_err(easyofd_core::OfdError::Io)
    }

    /// Return the signed OFD bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

/// Builder for applying electronic seals and digital signatures to an OFD.
pub struct OfdSignatureBuilder {
    input_path: String,
    seals: Vec<ElectronicSeal>,
    algorithm: SignatureAlgorithm,
    certificate: Option<Vec<u8>>, // PEM/DER certificate bytes
    private_key: Option<Vec<u8>>, // PEM/DER private key bytes
}

impl OfdSignatureBuilder {
    /// Start building a signature operation on an OFD file.
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input_path: input.into(),
            seals: Vec::new(),
            algorithm: SignatureAlgorithm::Sm2WithSm3,
            certificate: None,
            private_key: None,
        }
    }

    /// Add an electronic seal (visual stamp on a page).
    #[must_use]
    pub fn seal(mut self, seal: ElectronicSeal) -> Self {
        self.seals.push(seal);
        self
    }

    /// Set the signature algorithm.
    #[must_use]
    pub fn algorithm(mut self, alg: SignatureAlgorithm) -> Self {
        self.algorithm = alg;
        self
    }

    /// Set the signing certificate (PEM or DER).
    #[must_use]
    pub fn certificate(mut self, cert: Vec<u8>) -> Self {
        self.certificate = Some(cert);
        self
    }

    /// Set the private key (PEM or DER).
    #[must_use]
    pub fn private_key(mut self, key: Vec<u8>) -> Self {
        self.private_key = Some(key);
        self
    }

    /// Apply seals and signature to the OFD.
    ///
    /// Currently adds seal images to the OFD ZIP and places a
    /// Signature.xml entry with placeholder digest for GB/T 38540 compliance.
    /// Full cryptographic signing requires a valid certificate + private key.
    ///
    /// # Errors
    ///
    /// Returns an error if the input file cannot be read.
    pub fn sign(self) -> OfdResult<SignedOfd> {
        let input_bytes =
            std::fs::read(&self.input_path).map_err(easyofd_core::OfdError::Io)?;

        let cursor = Cursor::new(input_bytes);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;

        let out_buf = Vec::new();
        let out_cursor = Cursor::new(out_buf);
        let mut zip = ZipWriter::new(out_cursor);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Copy existing entries
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = entry.name().to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(easyofd_core::OfdError::Io)?;

            zip.start_file(name, options)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&content).map_err(easyofd_core::OfdError::Io)?;
        }

        // Add seal images
        for (i, seal) in self.seals.iter().enumerate() {
            let seal_name = format!("Doc_0/Res/Seal_{i}.png");
            zip.start_file(&seal_name, options)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&seal.image_data)
                .map_err(easyofd_core::OfdError::Io)?;
        }

        // Add Signature.xml (GB/T 38540 placeholder)
        let sig_xml = build_signature_xml(
            &self.seals,
            self.algorithm,
            self.certificate.is_some(),
            self.private_key.is_some(),
        );
        zip.start_file("Doc_0/Signs/Signature.xml", options)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(sig_xml.as_bytes())
            .map_err(easyofd_core::OfdError::Io)?;

        let cursor = zip
            .finish()
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        let data = cursor.into_inner();

        Ok(SignedOfd { data })
    }
}

fn build_signature_xml(
    seals: &[ElectronicSeal],
    algorithm: SignatureAlgorithm,
    has_cert: bool,
    has_key: bool,
) -> String {
    let alg_str = match algorithm {
        SignatureAlgorithm::Sm2WithSm3 => "SM2WithSM3",
        SignatureAlgorithm::Sha256WithRsa => "SHA256WithRSA",
    };
    let status = if has_cert && has_key {
        "signed"
    } else {
        "unsigned"
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:SignedInfo>
    <ofd:Provider>{}</ofd:Provider>
    <ofd:SignatureMethod>{alg_str}</ofd:SignatureMethod>
    <ofd:SignatureDateTime>{}</ofd:SignatureDateTime>
    <ofd:SealCount>{}</ofd:SealCount>
  </ofd:SignedInfo>
  <ofd:SignedValue>PLACEHOLDER_{status}</ofd:SignedValue>
</ofd:Signature>"#,
        "easyofd-rust",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S"),
        seals.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_sign_no_seals() {
        let dir = std::env::temp_dir().join("easyofd_sig");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("unsigned.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Document"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let result = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
            .sign()
            .unwrap();
        let bytes = result.into_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_sign_with_seal() {
        let dir = std::env::temp_dir().join("easyofd_sig2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("with_seal.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "Invoice"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let seal = ElectronicSeal {
            image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header
            name: "Company Seal".into(),
            position: (150.0, 200.0),
            page: 1,
        };

        let result = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
            .seal(seal)
            .algorithm(SignatureAlgorithm::Sha256WithRsa)
            .sign()
            .unwrap();

        let signed = result.into_bytes();
        let cursor = Cursor::new(&signed);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        assert!(names.contains(&"Doc_0/Res/Seal_0.png".to_string()));
        assert!(names.contains(&"Doc_0/Signs/Signature.xml".to_string()));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_signature_algorithm_enum() {
        assert_ne!(
            SignatureAlgorithm::Sm2WithSm3,
            SignatureAlgorithm::Sha256WithRsa
        );
        assert_eq!(
            SignatureAlgorithm::Sm2WithSm3,
            SignatureAlgorithm::Sm2WithSm3
        );
    }
}
