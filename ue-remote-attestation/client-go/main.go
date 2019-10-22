package main

import (
	"crypto/tls"
	"fmt"
	"io/ioutil"
	"log"
)

const (
	serverAddr = "localhost:4433"
	certPath   = "../pki/client.cert"
	keyPath    = "../pki/client.pkcs8"
)

func loadCert(certPath, keyPath string) (tls.Certificate, error) {
	certPEM, err := ioutil.ReadFile(certPath)
	if err != nil {
		return tls.Certificate{}, fmt.Errorf("failed to load cert: %w", err)
	}

	keyPEM, err := ioutil.ReadFile(keyPath)
	if err != nil {
		return tls.Certificate{}, fmt.Errorf("failed to load key: %w", err)
	}

	return tls.X509KeyPair(certPEM, keyPEM)
}

func main() {
	log.SetFlags(log.Lshortfile)
	println("Starting client-go")

	println("Connecting to ", serverAddr)

	cert, err := loadCert(certPath, keyPath)
	if err != nil {
		log.Fatalln(err)
	}

	config := &tls.Config{
		InsecureSkipVerify:    true,
		Certificates:          []tls.Certificate{cert},
		VerifyPeerCertificate: verify_mra_cert,
	}

	conn, err := tls.Dial("tcp", serverAddr, config)
	if err != nil {
		log.Fatalln(err)
	}
	defer conn.Close()

	if _, err := conn.Write([]byte("hello ue-ra go client")); err != nil {
		log.Fatalln(err)
	}

	var response [100]byte
	n, err := conn.Read(response[:])
	if err != nil {
		log.Fatalln(err)
	}

	println("server replied: ", string(response[:n]))
}
