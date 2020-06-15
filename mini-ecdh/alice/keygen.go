package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"fmt"
	"math/rand"
	"strings"
)

func main() {
	entropy := rand.New(rand.NewSource(123))

	priv, err := ecdsa.GenerateKey(elliptic.P256(), entropy)
	if err != nil {
		panic(err)
	}

	d := strings.ReplaceAll(fmt.Sprintf("%v", priv.D.Bytes()), " ", ",")
	fmt.Println("d =", d)

	x := strings.ReplaceAll(fmt.Sprintf("%v", priv.X.Bytes()), " ", ",")
	fmt.Println("x =", x)

	y := strings.ReplaceAll(fmt.Sprintf("%v", priv.Y.Bytes()), " ", ",")
	fmt.Println("y =", y)
}
