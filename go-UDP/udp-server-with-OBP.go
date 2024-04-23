package main

import (
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"net"
	"net/http"
	"time"
)

type DirectLoginToken struct {
	// defining struct variables note: struct needs Proper case field names
	Token string `json:"token"`
}

type CurrentUserId struct {
	UserId string `json:"user_id"`
}

// Metric represents the structure of the "metrics" array element in the JSON
type Metric struct {
	UserID                       string    `json:"user_id"`
	URL                          string    `json:"url"`
	Date                         time.Time `json:"date"`
	UserName                     string    `json:"user_name"`
	AppName                      string    `json:"app_name"`
	DeveloperEmail               string    `json:"developer_email"`
	ImplementedByPartialFunction string    `json:"implemented_by_partial_function"`
	ImplementedInVersion         string    `json:"implemented_in_version"`
	ConsumerID                   string    `json:"consumer_id"`
	Verb                         string    `json:"verb"`
	CorrelationID                string    `json:"correlation_id"`
	Duration                     int       `json:"duration"`
	SourceIP                     string    `json:"source_ip"`
	TargetIP                     string    `json:"target_ip"`
	ResponseBody                 string    `json:"response_body"`
}

type BasicMetric struct {
	UserID          string
	URL             string
	CorrelationID   string
	Duration        int
	SourceIP        string
	TargetIP        string
	PartialFunction string
}

// MetricsWrapper represents the structure of the root JSON object
type MetricsWrapper struct {
	Metrics []Metric `json:"metrics"`
}

func getMetrics(obpApiHost string, token string, offset int, limit int) (string, error) {

	fmt.Println(fmt.Sprintf("hello from getMetrics offset is %d, limit is %d ", offset, limit))

	// Create client
	client := &http.Client{}

	// defining a struct instance, we will put the token in this.
	var currentUserId CurrentUserId

	requestURL := fmt.Sprintf("%s/obp/v5.1.0/management/metrics?offset=%d&limit=%d&anon=false", obpApiHost, offset, limit)

	req, erry := http.NewRequest("GET", requestURL, nil)
	if erry != nil {
		fmt.Println("Failure : ", erry)
	}

	req.Header = http.Header{
		"Content-Type": {"application/json"},
		"DirectLogin":  {fmt.Sprintf("token=%s", token)},
	}

	before := time.Now()

	// Fetch Request
	resp, err1 := client.Do(req)

	after := time.Now()

	duration := after.Sub(before)

	if err1 != nil {
		fmt.Println("***** Failure when getting Metrics: ", err1)
	}

	// Read Response Body
	respBody, _ := io.ReadAll(resp.Body)

	// Display Results
	fmt.Println("getMetrics response Status : ", resp.Status)

	fmt.Println(fmt.Sprintf("getMetrics response Status was %s, offset was %d, limit was %d duration was %s", resp.Status, offset, limit, duration))

	//fmt.Println("response Headers : ", resp.Header)

	if resp.StatusCode != 200 {
		fmt.Println("getMetrics response Body : ", string(respBody))
		fmt.Println(fmt.Sprintf("offset was %d", offset))
		fmt.Println(fmt.Sprintf("limit was %d", limit))
	} else {

		// Unmarshal JSON into the struct
		var metricsWrapper MetricsWrapper
		err := json.Unmarshal([]byte(respBody), &metricsWrapper)
		if err != nil {
			fmt.Println("Error:", err)
		}

		// Accessing the data

		metrics := metricsWrapper.Metrics

		fmt.Println(fmt.Sprintf("Here are the %d Metrics records:", len(metrics)))

		for _, metric := range metrics {
			//fmt.Printf("User ID: %s\n", metric.UserID)
			fmt.Printf("URL: %s\n", metric.URL)
			fmt.Printf("Date: %s\n", metric.Date)
			fmt.Printf("User Name: %s\n", metric.UserName)
			fmt.Printf("App Name: %s\n", metric.AppName)
			//fmt.Printf("Developer Email: %s\n", metric.DeveloperEmail)
			fmt.Printf("Implemented By Partial Function: %s\n", metric.ImplementedByPartialFunction)
			fmt.Printf("Implemented In Version: %s\n", metric.ImplementedInVersion)
			fmt.Printf("Consumer ID: %s\n", metric.ConsumerID)
			fmt.Printf("Verb: %s\n", metric.Verb)
			fmt.Printf("Correlation ID: %s\n", metric.CorrelationID)
			fmt.Printf("Duration: %d\n", metric.Duration)
			//fmt.Printf("Source IP: %s\n", metric.SourceIP)
			//fmt.Printf("Target IP: %s\n", metric.TargetIP)
			//fmt.Printf("Response Body: %s\n", metric.ResponseBody)

		}

	}

	//fmt.Println("getMetrics response Body : ", string(respBody))

	// assuming respBody is the JSON equivelent of DirectLoginToken, put it in directLoginToken1
	//err2 := json.Unmarshal(respBody, &currentUserId)

	//if err2 != nil {
	//		fmt.Println(err2)
	//	}

	//	fmt.Println("Struct instance for currentUserId is:", currentUserId)
	//	fmt.Printf("UserId is %s \n", currentUserId.UserId)

	return currentUserId.UserId, nil

}

func getDirectLoginToken(obpApiHost string, username string, password string, consumerKey string) (string, error) {

	// defining a struct instance, we will put the token in this.
	var directLoginToken1 DirectLoginToken

	// Create client
	client := &http.Client{}

	// Create request path
	requestURL := fmt.Sprintf("%s/my/logins/direct", obpApiHost)

	// Nothing in the body
	req, err1 := http.NewRequest("POST", requestURL, nil)

	// Header
	//DirectLoginHeaderValue := fmt.Sprintf("username=%s, password=%s, consumer_key=%s", username, password, consumerKey)
	//fmt.Printf("DirectLoginHeaderValue : %s\n", DirectLoginHeaderValue)

	// Headers
	//req.Header.Add("DirectLogin", DirectLoginHeaderValue)
	//req.Header.Add("Content-Type", "application/json")

	req.Header = http.Header{
		"Content-Type": {"application/json"},
		"DirectLogin":  {fmt.Sprintf("username=%s, password=%s, consumer_key=%s", username, password, consumerKey)},
	}

	// Do the Request
	resp, err1 := client.Do(req)

	// var j interface{}
	// var err = json.NewDecoder(resp.Body).Decode(&j)
	// if err != nil {
	// 	panic(err)
	// }
	// fmt.Printf("%s", j)

	if err1 == nil {
		fmt.Println("We got a response from the http server. Will check Response Status Code...")
	} else {
		fmt.Println("We failed making the http request: ", err1)
		return "", err1
	}

	// Read Response Body
	respBody, _ := io.ReadAll(resp.Body)

	if resp.StatusCode == 201 {
		fmt.Printf("We got a 201 Response: %d \n", resp.StatusCode)
	} else {
		fmt.Printf("Hmm, Non ideal Response Status : %s \n", resp.Status)
		fmt.Printf("Response Body : %s \n", string(respBody))
		return "", errors.New("Non 201 Response")
	}

	//fmt.Println("response Headers : ", resp.Header)

	// assuming respBody is the JSON equivelent of DirectLoginToken, put it in directLoginToken1
	err2 := json.Unmarshal(respBody, &directLoginToken1)

	if err2 == nil {
		//fmt.Printf("I will return this token: %s \n", directLoginToken1.Token)
		return directLoginToken1.Token, nil
	} else {
		fmt.Printf("Struct instance is: %s", directLoginToken1)
		fmt.Printf("token is %s \n", directLoginToken1.Token)
		return "", err2
	}

}

func main() {

	var obpApiHost string
	var username string
	var password string
	var consumerKey string

	flag.StringVar(&obpApiHost, "obpapihost", "YOUR OBP HOST", "Provide an OBP host to test (include the protocol and port)")
	flag.StringVar(&username, "username", "YOUR USERNAME", "Username to access the service with")
	flag.StringVar(&password, "password", "YOUR PASSWORD", "Provide your password")
	flag.StringVar(&consumerKey, "consumer", "YOUR CONSUMER KEY", "Provide your consumer key")

	flag.Parse()

	fmt.Printf("I'm using the following values for -obpapihost -username -password -consumer \n")
	fmt.Println(obpApiHost)
	fmt.Println(username)
	fmt.Println(password)
	fmt.Println(consumerKey)

	myToken, dlTokenError := getDirectLoginToken(obpApiHost, username, password, consumerKey)

	if dlTokenError == nil {
		fmt.Printf("DirectLogin token i got: %s\n", myToken)
	} else {
		fmt.Printf("Hmm, getDirectLoginToken returned an error: %s WARNING \n", dlTokenError)
	}

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
		// This is the raw datafmt.Printf("Received: %s\n", buffer)

		// Convert the received data to uint64
		receivedData := uint64(buffer[0]) | uint64(buffer[1])<<8 | uint64(buffer[2])<<16 | uint64(buffer[3])<<24 |
			uint64(buffer[4])<<32 | uint64(buffer[5])<<40 | uint64(buffer[6])<<48 | uint64(buffer[7])<<56

		fmt.Printf("Received %d bytes from %s: %d\n", n, addr, receivedData)

		getMetrics(obpApiHost, myToken, 0, 1)

	}
}
