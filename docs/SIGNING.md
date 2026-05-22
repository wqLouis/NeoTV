# Windows Code Signing

This guide explains how to generate and configure code signing certificates for Windows builds in the CI/CD pipeline.

## Overview

The GitHub workflow (`.github/workflows/build.yml`) signs Windows executables automatically using **osslsigncode** on the Ubuntu runner. Signing requires two secrets and one optional variable:

| Name | Type | Purpose |
|------|------|---------|
| `WIN_CERT_BASE64` | Secret | Base64-encoded `.pfx` certificate file |
| `WIN_CERT_PASS` | Secret | Password for the `.pfx` |
| `WIN_SIGN_ENABLED` | Variable (optional) | Set to `true` to auto-sign on every push/PR |

## Option 1: Self-Signed Certificate (Testing Only)

A self-signed certificate will sign the binary, but Windows will show **"Unknown Publisher"** and SmartScreen may flag it. Use this for local testing or internal builds only.

### Using OpenSSL (Linux / macOS / WSL)

```bash
# 1. Generate a private key and self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem -out cert.pem \
  -days 365 -nodes \
  -subj "/CN=NeoTV"

# 2. Package into PKCS#12 (.pfx) — the format osslsigncode / signtool need
openssl pkcs12 -export \
  -out cert.pfx -inkey key.pem -in cert.pem \
  -passout pass:YOUR_PASSWORD_HERE

# 3. Base64-encode for GitHub secret
base64 -w0 cert.pfx > cert.pfx.b64

# 4. Copy the encoded string (macOS)
cat cert.pfx.b64 | pbcopy

# 4. Or print to terminal (Linux)
cat cert.pfx.b64
```

### Using PowerShell (Windows)

```powershell
# 1. Create a self-signed certificate
$cert = New-SelfSignedCertificate -Type Custom `
  -Subject "CN=NeoTV" `
  -KeyUsage DigitalSignature `
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3") `
  -CertStoreLocation "Cert:\CurrentUser\My"

# 2. Export as PFX
$pass = ConvertTo-SecureString "YOUR_PASSWORD_HERE" -AsPlainText -Force
Export-PfxCertificate -Cert $cert -FilePath cert.pfx -Password $pass

# 3. Base64-encode
$bytes = [IO.File]::ReadAllBytes("cert.pfx")
[Convert]::ToBase64String($bytes) | Set-Clipboard
Write-Host "Base64 cert copied to clipboard"
```

## Option 2: Proper Code Signing Certificate (Production)

A real certificate costs money but makes Windows trust your app — no SmartScreen warnings, shows your publisher name.

### Recommended Certificate Authorities

| Vendor | Approx. Cost | Notes |
|--------|-------------|-------|
| **DigiCert** | ~$300/yr | Most widely trusted, best timestamp support |
| **Sectigo** | ~$250/yr | Good alternative |
| **Certum** | ~$200/yr | Cheapest proper option |
| **Let's Encrypt** | Free ❌ | Cannot do code signing — TLS only |

### Requirements

- **OV (Organization Validation)** — your identity is verified, publisher name shown
- **EV (Extended Validation)** — higher trust level, but requires a USB hardware token (avoid for CI)
- **SHA-256** — must support SHA-256 and RFC 3161 timestamping (all modern CAs do)
- **Timestamping** — ensures the signature remains valid after the certificate expires

### Process

1. Purchase an OV code signing certificate from a CA
2. Generate a CSR locally:
   ```bash
   openssl req -new -newkey rsa:4096 -nodes \
     -keyout private-key.pem -out csr.pem \
     -subj "/C=US/ST=State/L=City/O=YourOrg/CN=YourOrg"
   ```
3. Submit the CSR to the CA — they will issue a `.cer` or `.crt`
4. Combine the private key and certificate into a PFX:
   ```bash
   openssl pkcs12 -export -out cert.pfx \
     -inkey private-key.pem -in your-certificate.crt \
     -passout pass:YOUR_PASSWORD_HERE

   base64 -w0 cert.pfx > cert.pfx.b64
   ```

## Setting Up GitHub Secrets

Once you have the base64-encoded PFX and its password:

1. Go to your repository on GitHub
2. Navigate to **Settings → Secrets and variables → Actions**
3. Add a **New repository secret**:

   | Name | Value |
   |------|-------|
   | `WIN_CERT_BASE64` | Paste the full base64 string from `cert.pfx.b64` |
   | `WIN_CERT_PASS` | The PFX password |

4. (Optional) Add a **New repository variable**:

   | Name | Value |
   |------|-------|
   | `WIN_SIGN_ENABLED` | `true` |

   Setting this variable enables signing on every push and PR. Without it, signing only runs when triggered manually via `workflow_dispatch` with `sign_windows=true`.

## Verification

### Sign Locally (Linux)

```bash
# Install osslsigncode
sudo apt install osslsigncode

# Sign
osslsigncode sign \
  -pkcs12 cert.pfx \
  -pass "YOUR_PASSWORD" \
  -h sha256 \
  -t http://timestamp.digicert.com \
  -in neotv.exe \
  -out neotv-signed.exe

mv neotv-signed.exe neotv.exe

# Verify
osslsigncode verify -in neotv.exe
```

### Sign Locally (Windows)

```powershell
# Sign with signtool (from Windows SDK)
signtool sign /fd SHA256 /a /f cert.pfx /p "YOUR_PASSWORD" `
  /tr http://timestamp.digicert.com /td SHA256 neotv.exe

# Verify
signtool verify /pa neotv.exe
```

## How It Works in CI

The GitHub workflow performs these steps:

```
WIN_CERT_BASE64 (secret)
    │
    ▼  base64 -d
/tmp/signing_cert.pfx
    │
    ▼  osslsigncode sign -pkcs12 ... -pass "$PASS" \
    │     -h sha256 -t http://timestamp.digicert.com
    │
    ▼
neotv-signed.exe  →  rename →  neotv.exe
    │
    ▼  osslsigncode verify
✅ Signed & verified
    │
    ▼
Uploaded as artifact: neotv-windows-x86_64
```

## Troubleshooting

### "No such file or directory: osslsigncode"
The Ubuntu runner should have it pre-installed via `apt`. If missing, the workflow installs it.

### Signing fails with PKCS12 error
- Make sure the PFX password is correct
- Ensure the PFX was exported correctly: `openssl pkcs12 -export -out cert.pfx -inkey key.pem -in cert.pem`

### "The timestamp server could not be reached"
- The default timestamp server is `http://timestamp.digicert.com`
- If it's blocked in your network, change the `-t` URL to an alternative:
  - `http://timestamp.sectigo.com`
  - `http://ts.ssl.com`
  - `http://timestamp.comodoca.com`

### Self-signed cert still shows "Unknown Publisher"
This is expected. Self-signed certificates are not trusted by Windows. Only a certificate from a trusted CA (DigiCert, Sectigo, etc.) will show your publisher name.
