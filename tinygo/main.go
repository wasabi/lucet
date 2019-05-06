package main

import (
	"fmt"
)

//go:export random
func random(a []byte)

func main() {
	// foo := time.Now()
	// fmt.Println("does this work?", foo)

	byt := make([]byte, 10)

	random(byt)
	fmt.Println(byt)
	go func() {
		fmt.Println("within a goroutine")
	}()
	// testSetGetenv()
	// testHTTP()
	// time.Sleep(time.Millisecond * 5)
	//
	fmt.Println("after sleep")
}
