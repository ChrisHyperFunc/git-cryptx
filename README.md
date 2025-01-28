# git-cryptx

English | [简体中文](README_ZH.md)

Automatically encrypt/decrypt sensitive files in Git repositories.
https://git-cryptx.201945.xyz

## Features

- 🔒 Transparent file encryption/decryption
- 🔄 Seamless Git workflow integration
- 🎯 Precise file pattern matching
- 👥 Team collaboration support
- 💻 Cross-platform support

## Installation

## Quick Start

1. Initialize repository:
git-cryptx init

2. Set encryption key:
git-cryptx set-key <your-key>

3. Configure files to encrypt (edit .gitattributes):
```
.secret filter=git-cryptx diff=git-cryptx
config/.key filter=git-cryptx diff=git-cryptx
sensitive/ filter=git-cryptx diff=git-cryptx
```

## Commands

- `init`: Initialize git-cryptx
- `set-key <key>`: Add encryption key
- `rm-key`: Remove encryption key
- `status`: Show encryption status

## How It Works

git-cryptx uses Git's filter mechanism to automatically encrypt and decrypt files:

1. When files are added to Git, the clean filter encrypts content
2. When files are checked out, the smudge filter decrypts content
3. Files remain in plaintext in working directory
4. Files remain encrypted in Git repository

## Security Notes

- Uses AES-256-GCM for encryption
- Keys stored in .git/cryptx directory
- Supports file integrity verification
- Encrypted files marked with magic number

## FAQ

Q: How to share keys with team members?
A: Share the .git/cryptx/keys/global_ase_key file through a secure channel.

Q: How to view differences in encrypted files?
A: git-cryptx supports viewing plaintext differences directly using regular git diff.

## Team Collaboration

When a new team member joins the project, follow these steps:

1. Clone the repository:
```bash
git clone <repository-url>
```

2. Initialize git-cryptx:
```bash
git-cryptx init
```

3. Obtain the key file from other team members:
   - Get the `.git/cryptx/keys/global_ase_key` file
   - Place it in the same location in your local repository
   - Or git-cryptx git-cryptx set-key <key>

4. Check configuration status:
```bash
git-cryptx status
```

5. Update working directory files:
```bash
# Clean working directory
git clean -fd
# Checkout files to trigger decryption
git checkout .
```

Important notes:
- Transfer the key file through secure channels (encrypted email, secure messaging, etc.)
- Never commit the key file to the Git repository
- Each cloned repository needs its own key configuration
- If files appear encrypted, the key is not properly configured

## Contributing

Pull requests and issues are welcome.

## License

MIT License