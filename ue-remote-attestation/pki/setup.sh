#!/bin/sh

# DO NOT USE THE OPENSSL ON MACOS, IT'S BROKEN.

# 1. Generate CA private key
echo "> Generating CA private key"
openssl ecparam -genkey -name prime256v1 -out ca.key
echo "> Done generating CA private key"

# 2. Generate CA cert
echo "> Generate CA cert"
openssl req -x509 -new -SHA256 -nodes -key ca.key -days 3650 -subj /C=CN/ST=SH/O=ORG -out ca.cert
echo "> Done generate CA cert"

# 3. Generate Client private key
echo "> Generate client private key"
openssl ecparam -genkey -name prime256v1 -out client.key
echo "> Done generate client private key"

# 4. Export the keys to pkcs8 unencrypted format
echo "> Export the keys to pkcs8 unencrypted format"
openssl pkcs8 -topk8 -nocrypt -in client.key -out client.pkcs8
echo "> Done export the keys to pkcs8 unencrypted format"

# 5. Generate Client CSR
echo "> Generate Client CSR"
openssl req -new -SHA256 -key client.key -nodes -subj /C=CN/ST=SH/O=ORG -out client.csr
echo "> Done generate Client CSR"

# 6. Generate Client Cert 
echo "> Generate Client Cert"
openssl x509 -req -extfile ssl.conf -extensions v3_client \
    -days 3650 -in client.csr -CA ca.cert -CAkey ca.key -CAcreateserial -out client.cert
echo "> Done generate Client Cert"

# 7. Intel CA report signing pem. Download and uncompress:
# IntelRootCA=Intel_SGX_Attestation_RootCA.pem
# curl -o ${IntelRootCA} https://certificates.trustedservices.intel.com/${IntelRootCA}