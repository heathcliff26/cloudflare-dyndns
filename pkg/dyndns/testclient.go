package dyndns

import "fmt"

// This is a stub implementation of Client, it is only meant to be used for testing
type testClient struct {
	data *ClientData
	// Variables used to control and check Update during tests
	UpdateCount int
	FailUpdate  bool
}

// Create a new testClient, fails if the token is empty
func NewTestClient(token string, proxy bool) (Client, error) {
	if token == "" {
		return nil, ErrMissingToken{}
	}
	return &testClient{
		data:       NewClientData(proxy),
		FailUpdate: false,
	}, nil
}

// Give Access to ClientData
func (c *testClient) Data() *ClientData {
	return c.data
}

// Stub implementation, does initial check regarding IP and domains
func (c *testClient) Update() error {
	err := c.Data().CheckData()
	if err != nil {
		return err
	}
	if c.FailUpdate {
		return fmt.Errorf("i'm instructed to throw an error")
	}
	c.UpdateCount++

	return nil
}
