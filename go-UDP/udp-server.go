package main

import (
	"fmt"
	"net"
)

func main() {
	// Define the UDP address to listen on
	addr, err := net.ResolveUDPAddr("udp", ":7001")
	if err != nil {
		fmt.Println("Error resolving address:", err)
		return
	}

	// Create a UDP connection
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		fmt.Println("Error listening:", err)
		return
	}
	defer conn.Close()

	fmt.Println("UDP server is listening on", addr)

	// Buffer to store incoming data
	buffer := make([]byte, 8)

	for {
		// Read data from the connection
		n, addr, err := conn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Println("Error reading data:", err)
			continue

		}
		fmt.Printf("Received: %s\n", buffer)

		// Convert the received data to uint64
		receivedData := uint64(buffer[0]) | uint64(buffer[1])<<8 | uint64(buffer[2])<<16 | uint64(buffer[3])<<24 |
			uint64(buffer[4])<<32 | uint64(buffer[5])<<40 | uint64(buffer[6])<<48 | uint64(buffer[7])<<56

		fmt.Printf("Received %d bytes from %s: %d\n", n, addr, receivedData)

	}
}
