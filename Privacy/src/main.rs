use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey as RsaPublicKey, RsaPrivateKey, RsaPublicKey as _};
use std::collections::HashMap;
use eframe::egui;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng as AesOsRng},
    Aes256Gcm,
    Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Structure to hold user information
#[derive(Clone)]
struct User {
    username: String,
    keypair: Keypair,                    // For signatures
    rsa_private: RsaPrivateKey,          // For encryption
    rsa_public: RsaPublicKey,            // For encryption
}

// Structure to hold an encrypted message
#[derive(Clone)]
struct EncryptedMessage {
    encrypted_data: Vec<u8>,             // The encrypted message
    signature: Signature,                // Signature of the original message
    sender_public: PublicKey,            // Sender's public key for verification
    symmetric_key: Vec<u8>,              // Encrypted symmetric key
    nonce: Vec<u8>,                      // Nonce for AES-GCM
}

// Main application state
struct SignatureApp {
    users: HashMap<String, User>,
    current_user: Option<String>,
    recipient: String,
    message: String,
    encrypted_messages: Vec<(String, EncryptedMessage)>,
    decrypted_messages: Vec<(String, String)>,
    new_username: String,
}

impl Default for SignatureApp {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            current_user: None,
            recipient: String::new(),
            message: String::new(),
            encrypted_messages: Vec::new(),
            decrypted_messages: Vec::new(),
            new_username: String::new(),
        }
    }
}

impl SignatureApp {
    // Create a new user with keypair
    fn create_user(&mut self, username: String) {
        let mut csprng = OsRng;
        
        // Generate Ed25519 keypair for signatures
        let keypair = Keypair::generate(&mut csprng);
        
        // Generate RSA keypair for encryption
        let rsa_private = RsaPrivateKey::new(&mut csprng, 2048).expect("Failed to generate RSA key");
        let rsa_public = rsa_private.to_public_key();
        
        let user = User {
            username: username.clone(),
            keypair,
            rsa_private,
            rsa_public,
        };
        
        self.users.insert(username, user);
    }
    
    // Encrypt and sign a message
    fn encrypt_message(&self, sender: &User, recipient: &User, message: &str) -> EncryptedMessage {
        // Generate a random symmetric key
        let symmetric_key = Aes256Gcm::generate_key(&mut AesOsRng);
        
        // Create cipher
        let cipher = Aes256Gcm::new(&symmetric_key);
        let nonce = Aes256Gcm::generate_nonce(&mut AesOsRng);
        
        // Encrypt the message using AES-GCM
        let encrypted_data = cipher
            .encrypt(&nonce, message.as_bytes().as_ref())
            .expect("Encryption failed");
        
        // Sign the original message
        let signature = sender.keypair.sign(message.as_bytes());
        
        // Encrypt the symmetric key with recipient's RSA public key
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let encrypted_symmetric_key = recipient
            .rsa_public
            .encrypt(&mut OsRng, padding, &symmetric_key)
            .expect("Failed to encrypt symmetric key");
        
        EncryptedMessage {
            encrypted_data,
            signature,
            sender_public: sender.keypair.public,
            symmetric_key: encrypted_symmetric_key,
            nonce: nonce.to_vec(),
        }
    }
    
    // Decrypt and verify a message
    fn decrypt_message(&self, recipient: &User, message: &EncryptedMessage) -> Option<String> {
        // Decrypt the symmetric key using recipient's private key
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let symmetric_key = recipient
            .rsa_private
            .decrypt(padding, &message.symmetric_key)
            .ok()?;
        
        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&symmetric_key).ok()?;
        let nonce = Nonce::from_slice(&message.nonce);
        
        // Decrypt the message
        let decrypted_data = cipher
            .decrypt(nonce, message.encrypted_data.as_ref())
            .ok()?;
        
        let decrypted_message = String::from_utf8(decrypted_data).ok()?;
        
        // Verify the signature
        message
            .sender_public
            .verify(
                decrypted_message.as_bytes(),
                &message.signature,
            )
            .ok()?;
        
        Some(decrypted_message)
    }
}

impl eframe::App for SignatureApp {
    
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Digital Signature System",
        options,
        Box::new(|_cc| Box::<SignatureApp>::default()),
    )
}