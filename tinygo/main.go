package main

import (
	"fmt"
	"time"
)

// //go:export random
// func random(a []byte)

func main() {
	foo := time.Now()
	fmt.Println("does this work?", foo)

	// byt := make([]byte, 10)

	// random(byt)
	// fmt.Println(byt)
	go func() {
		time.Sleep(time.Millisecond * 5)
		fmt.Println("within a goroutine")
	}()
	// testSetGetenv()
	// testHTTP()

	//
	fmt.Println("before some sleep")
	time.Sleep(time.Millisecond * 10)
	fmt.Println("after some sleep")
}
