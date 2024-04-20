package main

import (
	"encoding/json"
	"fmt"
	"net"
	"os"
	"time"
)

type Message struct {
	Text string `json:"text"`
}

func main() {
	// Server address to send UDP packets to
	serverAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:7001")
	if err != nil {
		fmt.Println("Error resolving address:", err)
		os.Exit(1)
	}

	// Create a UDP connection
	conn, err := net.DialUDP("udp", nil, serverAddr)
	if err != nil {
		fmt.Println("Error connecting:", err)
		os.Exit(1)
	}
	defer conn.Close()

	// Create a JSON message

	// Get the current time with milliseconds
	currentTime := time.Now().Format("2006-01-02 15:04:05.000")

	message := Message{
		Text: currentTime,
	}
	jsonBytes, err := json.Marshal(message)
	if err != nil {
		fmt.Println("Error marshalling JSON:", err)
		os.Exit(1)
	}

	// Send the JSON string as UDP packet
	_, err = conn.Write(jsonBytes)
	if err != nil {
		fmt.Println("Error sending data:", err)
		os.Exit(1)
	}

	fmt.Println("Message sent successfully!")
}
