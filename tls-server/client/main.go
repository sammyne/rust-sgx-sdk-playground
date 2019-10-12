package main

import (
	"crypto/tls"
	"crypto/x509"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
)

func loadRootCA(path string) (*x509.CertPool, error) {
	caCertPEM, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}

	// Configure a client to trust the server
	certPool := x509.NewCertPool()
	if ok := certPool.AppendCertsFromPEM(caCertPEM); !ok {
		return nil, errors.New("failed to append CA certs")
	}

	return certPool, nil
}

func main() {
	rootCAs, err := loadRootCA("../pki/end.fullchain")
	if err != nil {
		panic(err)
	}

	config := tls.Config{RootCAs: rootCAs}

	//注意这里要使用证书中包含的主机名称
	conn, err := tls.Dial("tcp", "localhost:4433", &config)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	fmt.Println("connect to", conn.RemoteAddr())

	//fmt.Printf("%#v\n", conn.ConnectionState())

	if _, err := io.WriteString(conn, "hello"); err != nil {
		panic(err)
	}

	var buf [1024]byte
	ell, err := conn.Read(buf[:])
	if err != nil {
		panic(err)
	}

	fmt.Println("Receive From Server:", string(buf[:ell]))
}
