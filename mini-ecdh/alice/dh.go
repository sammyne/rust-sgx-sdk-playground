// +build ignore

package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"fmt"
	"math/big"
	"math/rand"
)

func main() {
	entropy := rand.New(rand.NewSource(123))

	priv, err := ecdsa.GenerateKey(elliptic.P256(), entropy)
	if err != nil {
		panic(err)
	}

	// update x and y according to that output by enclave
	x := new(big.Int).SetBytes([]byte{
		29, 243, 184, 75, 173, 197, 181, 137, 118, 12, 64, 216, 97, 17, 251, 248, 84, 243, 148, 82, 15, 168, 228, 4, 89, 230, 232, 202, 241, 38, 248, 184,
	})
	y := new(big.Int).SetBytes([]byte{
		238, 137, 120, 33, 157, 219, 215, 53, 167, 204, 138, 11, 217, 16, 193, 201, 179, 18, 204, 48, 71, 106, 137, 177, 231, 146, 209, 142, 55, 4, 166, 14,
	})

	xx, yy := priv.Curve.ScalarMult(x, y, priv.D.Bytes())
	fmt.Println("xx =", xx.Bytes())
	fmt.Println("yy =", yy.Bytes())

}
