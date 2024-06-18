<!-- cspell: words outpubkey mycommname genpkey -->

# Certificate and Keys

Hermes utilizes the [x.509] standard as the foundation for signing and verification procedures.
Users can either have their key pair certified by a [Certificate Authority][CA] (CA) or they can generate a self-signed certificate.
The signing procedure itself does not differentiate between the two.

By default, Hermes does not enforce any strict rules for certificates and will only issue a warning to the user.
This allows for flexibility in the certificate management process.

## Signing Algorithm

The cryptography algorithm used for signing in Hermes is [Ed25519],
which is an EdDSA signature scheme that utilizes [SHA-512] (SHA-2) and [Curve25519].
[Ed25519] provides strong security and is widely supported.

## Keys and Certificate Format

Private/Public keys or certificates should be stored in the [PEM] format.
This format is defined in [RFC 7468] and is widely recognized and supported.
It is the preferred format for storing and distributing keys and certificates.

Private key example:

```PEM
-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIP1iI3LF7h89yY6QZmhDp4Y5FmTQ4oasbz2lEiaqqTzV
-----END PRIVATE KEY-----
```

Public key example:

```PEM
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAtFuCleJwHS28jUCT+ulLl5c1+MXhehhDz2SimOhmWaI=
-----END PUBLIC KEY-----
```

Certificate example:

```PEM
-----BEGIN CERTIFICATE-----
MIIByzCCAX2gAwIBAgIUXSsFYOzs6TCwKZ4ASbby0XbUuI8wBQYDK2VwMFsxCzAJ
BgNVBAYTAnVhMQowCAYDVQQIDAFhMQowCAYDVQQHDAFhMQowCAYDVQQKDAFhMQow
CAYDVQQLDAFhMQowCAYDVQQDDAFhMRAwDgYJKoZIhvcNAQkBFgFhMB4XDTI0MDYx
MDExMDEzMloXDTI0MDkxODExMDEzMlowWzELMAkGA1UEBhMCdWExCjAIBgNVBAgM
AWExCjAIBgNVBAcMAWExCjAIBgNVBAoMAWExCjAIBgNVBAsMAWExCjAIBgNVBAMM
AWExEDAOBgkqhkiG9w0BCQEWAWEwKjAFBgMrZXADIQC0W4KV4nAdLbyNQJP66UuX
lzX4xeF6GEPPZKKY6GZZoqNTMFEwHQYDVR0OBBYEFGAwFd05R9zFSYqT3C9xvFgI
zwKhMB8GA1UdIwQYMBaAFGAwFd05R9zFSYqT3C9xvFgIzwKhMA8GA1UdEwEB/wQF
MAMBAf8wBQYDK2VwA0EAqGmRv75DXvjtLrvmuc5KCAuE4jJ2AHCcImWzOYj+m4pT
n6SS3ECqDuMa5Pz3NKue2fsvNucerJgbn6lKlRJ1BQ==
-----END CERTIFICATE-----
```

## Certificate and keys generation

For key generation and certificate signing we'll use [OpenSSl] command-line tools.

To generate private and public keys run the following:

```shell
openssl genpkey -algorithm=ED25519 -out=private.pem -outpubkey=public.pem
```

After that a self-signed certificate could be generated by:

```shell
openssl req -new -x509 -key=private.pem -out=cert.pem -config=x509_cert.config
```

Where `x509_cert.config` could look like this

```CONF
[req]
prompt                 = no
days                   = 365
distinguished_name     = req_distinguished_name


[req_distinguished_name]
countryName            = AB
stateOrProvinceName    = CD
localityName           = EFG_HIJ
organizationName       = MyOrg
organizationalUnitName = MyOrgUnit
commonName             = mycommname.com
emailAddress           = emailaddress@myemail.com
```

To double check the content of the certificate and certificate details run:

```shell
openssl x509 -text -in=cert.pem                 
```

[x.509]: https://datatracker.ietf.org/doc/html/rfc5280
[CA]: https://en.wikipedia.org/wiki/Certificate_authority
[ED25519]: https://en.wikipedia.org/wiki/EdDSA#ed25519
[SHA-512]: https://en.wikipedia.org/wiki/SHA-512
[Curve25519]: https://en.wikipedia.org/wiki/Curve25519
[PEM]: https://en.wikipedia.org/wiki/Privacy-Enhanced_Mail
[RFC 7468]: https://tools.ietf.org/html/rfc7468
[OpenSSL]: https://www.openssl.org