# Generate TLS certificates

## Creating a new CA certificate
```
<!-- Generate CA key -->
openssl genrsa -des3 -out CA.key 2048
<!-- Generate CA cert from key -->
openssl req -x509 -new -nodes -key CA.key -sha256 -days 1825 -out CA.pem
<!-- Add certificate to mac keychain -->
sudo security add-trusted-cert -d -r trustRoot -k "/Library/Keychains/System.keychain" CA.pem
```

## Creating a new certificates
1. Create a CONFIG file
   ```
    api_server.cnf
    FQDN = 127.0.0.1
    ORGNAME = Jeloby
    ALTNAMES = DNS:$FQDN,DNS:localhost   # , DNS:bar.example.org , DNS:www.foo.example.org

    [ req ]
    default_bits = 2048
    default_md = sha256
    prompt = no
    encrypt_key = no
    distinguished_name = dn
    req_extensions = req_ext

    [ dn ]
    C = ZA
    O = $ORGNAME
    CN = $FQDN

    [ req_ext ]
    subjectAltName = $ALTNAMES
    FQDN = 127.0.0.1
    ORGNAME = Jeloby
    ALTNAMES = DNS:$FQDN,DNS:localhost   # , DNS:bar.example.org , DNS:www.foo.example.org

    [ req ]
    default_bits = 2048
    default_md = sha256
    prompt = no
    encrypt_key = no
    distinguished_name = dn
    req_extensions = req_ext

    [ dn ]
    C = ZA
    O = $ORGNAME
    CN = $FQDN

    [ req_ext ]
    subjectAltName = $ALTNAMES
    ```

### Create standard Certificates
1. Generate a CSR and KEY file
    ```
    openssl req -new -config api_server.cnf -keyout ./private/api_server.key \
    -out api_server.csr
    ```
2.  Generate Certificate
    ```
    openssl x509 -req -days 365 -in api_server.csr \
    -signkey ./private/api_server.key -sha256 \
    -CA ./ca/CA.pem -CAkey ./ca/CA.key -CAcreateserial \
    -out ./certs/api_server.crt

### Generate .pem Certificates
1.  Generate a CSR and KEY file
    ```
    openssl req -new -config api_server.cnf -keyout ./private/api_server.key.pem \
    -out api_server.csr
    ```
2.  Generate Certificate
    ```
    openssl x509 -req -days 825 -in api_server.csr \
        -signkey ./private/api_server.key.pem -sha256 \
        -CA ./ca/CA.pem -CAkey ./ca/CA.key -CAcreateserial \
        -out ./certs/api_server.crt.pem
    ```
3.  Generate PFX file
    ```
    openssl pkcs12 -export -out certificate.pfx -inkey ./private/api_server.key -in ./certs/api_server.crt -CA ./ca/CA.pem -CAkey ./ca/CA.key -CAcreateserial
    ```