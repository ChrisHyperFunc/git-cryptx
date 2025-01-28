use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("加密错误: {0}")]
    EncryptError(String),
    #[error("密钥错误: {0}")]
    KeyError(String),
}

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    // 魔数标记，用于识别加密文件
    const MAGIC_HEADER: &'static [u8] = b"GITENC";
    // nonce 长度（12字节是 AES-GCM 的推荐值）
    const NONCE_SIZE: usize = 12;

    pub fn is_encrypted(content: &[u8]) -> bool {
        content.starts_with(Self::MAGIC_HEADER)
    }

    pub fn new(key: &[u8]) -> Result<Self, CryptoError> {
        if key.len() < 8 {
            return Err(CryptoError::KeyError("密钥长度不足".to_string()));
        }

        // 使用 SHA-256 扩展密钥到32字节
        let mut hasher = Sha256::new();
        hasher.update(key);
        let key = hasher.finalize();

        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| CryptoError::KeyError(e.to_string()))?;

        Ok(Self { cipher })
    }

    // 新增：根据文件内容生成确定性nonce
    fn generate_deterministic_nonce(&self, data: &[u8]) -> [u8; Self::NONCE_SIZE] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        // 使用哈希的前12字节作为nonce
        let mut nonce = [0u8; Self::NONCE_SIZE];
        nonce.copy_from_slice(&hash[..Self::NONCE_SIZE]);
        nonce
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // 使用确定性nonce替代随机nonce
        let nonce_bytes = self.generate_deterministic_nonce(data);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 计算输出大小：魔数 + nonce + 加密数据
        let mut output =
            Vec::with_capacity(Self::MAGIC_HEADER.len() + Self::NONCE_SIZE + data.len() + 16);

        // 写入魔数
        output.extend_from_slice(Self::MAGIC_HEADER);
        // 写入 nonce
        output.extend_from_slice(&nonce_bytes);

        // 加密数据
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| CryptoError::EncryptError(e.to_string()))?;

        // 写入加密数据
        output.extend(ciphertext);
        Ok(output)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // 验证魔数
        if !Self::is_encrypted(data) {
            return Err(CryptoError::EncryptError("不是加密的数据".to_string()));
        }

        // 提取 nonce 和加密数据
        let data = &data[Self::MAGIC_HEADER.len()..];
        if data.len() < Self::NONCE_SIZE {
            return Err(CryptoError::EncryptError("数据格式错误".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(Self::NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密数据
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::EncryptError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = b"test-key-12345";
        let data = b"Hello, World!";

        let encryptor = Encryptor::new(key).unwrap();

        // 加密
        let encrypted = encryptor.encrypt(data).unwrap();
        assert!(Encryptor::is_encrypted(&encrypted));

        // 解密
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_different_nonce() {
        let key = b"test-key-12345";
        let data = b"Hello, World!";
        let encryptor = Encryptor::new(key).unwrap();

        // 两次加密应该产生不同的密文
        let encrypted1 = encryptor.encrypt(data).unwrap();
        let encrypted2 = encryptor.encrypt(data).unwrap();
        assert_ne!(encrypted1, encrypted2);

        // 但解密后应该得到相同的明文
        let decrypted1 = encryptor.decrypt(&encrypted1).unwrap();
        let decrypted2 = encryptor.decrypt(&encrypted2).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(decrypted1, data);
    }

    #[test]
    fn test_deterministic_encryption() {
        let key = b"test-key-12345";
        let data = b"Hello, World!";
        let encryptor = Encryptor::new(key).unwrap();

        // 相同数据应该产生相同的密文
        let encrypted1 = encryptor.encrypt(data).unwrap();
        let encrypted2 = encryptor.encrypt(data).unwrap();
        assert_eq!(encrypted1, encrypted2);

        // 解密后应该得到原始数据
        let decrypted = encryptor.decrypt(&encrypted1).unwrap();
        assert_eq!(decrypted, data);
    }
}
