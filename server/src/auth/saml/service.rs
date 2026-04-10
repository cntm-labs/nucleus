use crate::core::error::AppError;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User info extracted from a SAML assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlUserInfo {
    pub name_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Attribute mapping configuration for a SAML connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeMapping {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

impl Default for AttributeMapping {
    fn default() -> Self {
        Self {
            email: "email".to_string(),
            first_name: "firstName".to_string(),
            last_name: "lastName".to_string(),
        }
    }
}

pub struct SamlService;

impl SamlService {
    /// Parse a base64-encoded SAML Response and extract user attributes.
    ///
    /// NOTE: This is a foundation implementation — it parses assertions and
    /// extracts attributes but does NOT verify XML signatures. Signature
    /// verification should be added before production use with untrusted IdPs.
    pub fn parse_response(
        saml_response_b64: &str,
        attribute_mapping: &AttributeMapping,
    ) -> Result<SamlUserInfo, AppError> {
        let decoded = STANDARD.decode(saml_response_b64).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Invalid base64 SAML response: {}", e))
        })?;

        let xml = String::from_utf8(decoded).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Invalid UTF-8 in SAML response: {}", e))
        })?;

        Self::parse_xml(&xml, attribute_mapping)
    }

    /// Parse SAML Response XML and extract user info.
    fn parse_xml(
        xml: &str,
        attribute_mapping: &AttributeMapping,
    ) -> Result<SamlUserInfo, AppError> {
        let mut reader = Reader::from_str(xml);

        let mut name_id = None;
        let mut attributes: HashMap<String, String> = HashMap::new();

        // State tracking
        let mut in_name_id = false;
        let mut current_attr_name: Option<String> = None;
        let mut in_attr_value = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match local_name.as_str() {
                        "NameID" => {
                            in_name_id = true;
                        }
                        "Attribute" => {
                            // Extract Name attribute
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"Name" {
                                    current_attr_name =
                                        Some(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                        "AttributeValue" => {
                            if current_attr_name.is_some() {
                                in_attr_value = true;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_name_id {
                        name_id = Some(text);
                        in_name_id = false;
                    } else if in_attr_value {
                        if let Some(ref name) = current_attr_name {
                            attributes.insert(name.clone(), text);
                        }
                        in_attr_value = false;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match local_name.as_str() {
                        "NameID" => in_name_id = false,
                        "Attribute" => current_attr_name = None,
                        "AttributeValue" => in_attr_value = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(AppError::Internal(anyhow::anyhow!(
                        "Failed to parse SAML XML: {}",
                        e
                    )));
                }
                _ => {}
            }
        }

        let name_id = name_id.ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("No NameID found in SAML assertion"))
        })?;

        Ok(SamlUserInfo {
            name_id,
            email: attributes.get(&attribute_mapping.email).cloned(),
            first_name: attributes.get(&attribute_mapping.first_name).cloned(),
            last_name: attributes.get(&attribute_mapping.last_name).cloned(),
        })
    }

    /// Generate SP metadata XML for IdP configuration.
    pub fn generate_metadata(entity_id: &str, acs_url: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{entity_id}">
  <SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true"
    protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
      Location="{acs_url}" index="0" isDefault="true"/>
  </SPSSODescriptor>
</EntityDescriptor>"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_saml_response_xml() -> &'static str {
        r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
  <saml:Assertion>
    <saml:Subject>
      <saml:NameID>user@example.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="email">
        <saml:AttributeValue>user@example.com</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="firstName">
        <saml:AttributeValue>Jane</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="lastName">
        <saml:AttributeValue>Doe</saml:AttributeValue>
      </saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"#
    }

    #[test]
    fn parse_xml_extracts_user_info() {
        let mapping = AttributeMapping::default();
        let result = SamlService::parse_xml(sample_saml_response_xml(), &mapping).unwrap();

        assert_eq!(result.name_id, "user@example.com");
        assert_eq!(result.email, Some("user@example.com".to_string()));
        assert_eq!(result.first_name, Some("Jane".to_string()));
        assert_eq!(result.last_name, Some("Doe".to_string()));
    }

    #[test]
    fn parse_response_decodes_base64() {
        let xml = sample_saml_response_xml();
        let encoded = STANDARD.encode(xml);
        let mapping = AttributeMapping::default();

        let result = SamlService::parse_response(&encoded, &mapping).unwrap();

        assert_eq!(result.name_id, "user@example.com");
        assert_eq!(result.email, Some("user@example.com".to_string()));
    }

    #[test]
    fn parse_response_rejects_invalid_base64() {
        let mapping = AttributeMapping::default();
        let result = SamlService::parse_response("not-valid-base64!!!", &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn parse_xml_custom_attribute_mapping() {
        let xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
  <saml:Assertion>
    <saml:Subject>
      <saml:NameID>user123</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="mail">
        <saml:AttributeValue>custom@example.com</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="givenName">
        <saml:AttributeValue>Custom</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="sn">
        <saml:AttributeValue>User</saml:AttributeValue>
      </saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"#;

        let mapping = AttributeMapping {
            email: "mail".to_string(),
            first_name: "givenName".to_string(),
            last_name: "sn".to_string(),
        };

        let result = SamlService::parse_xml(xml, &mapping).unwrap();

        assert_eq!(result.name_id, "user123");
        assert_eq!(result.email, Some("custom@example.com".to_string()));
        assert_eq!(result.first_name, Some("Custom".to_string()));
        assert_eq!(result.last_name, Some("User".to_string()));
    }

    #[test]
    fn parse_xml_missing_attributes_returns_none() {
        let xml = r#"<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">
  <saml:Assertion>
    <saml:Subject>
      <saml:NameID>user@example.com</saml:NameID>
    </saml:Subject>
    <saml:AttributeStatement>
      <saml:Attribute Name="email">
        <saml:AttributeValue>user@example.com</saml:AttributeValue>
      </saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>"#;

        let mapping = AttributeMapping::default();
        let result = SamlService::parse_xml(xml, &mapping).unwrap();

        assert_eq!(result.email, Some("user@example.com".to_string()));
        assert_eq!(result.first_name, None);
        assert_eq!(result.last_name, None);
    }

    #[test]
    fn generate_metadata_includes_entity_id_and_acs() {
        let metadata = SamlService::generate_metadata(
            "https://nucleus.example.com",
            "https://nucleus.example.com/api/v1/auth/saml/callback",
        );

        assert!(metadata.contains("entityID=\"https://nucleus.example.com\""));
        assert!(
            metadata.contains("Location=\"https://nucleus.example.com/api/v1/auth/saml/callback\"")
        );
        assert!(metadata.contains("WantAssertionsSigned=\"true\""));
        assert!(metadata.contains("HTTP-POST"));
    }
}
