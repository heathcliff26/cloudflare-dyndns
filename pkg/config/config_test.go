package config

import (
	"log/slog"
	"reflect"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestValidConfigs(t *testing.T) {
	c1 := Config{
		LogLevel: "info",
		Server: ServerConfig{
			Port:    8080,
			Domains: []string{"example.org", "example.net"},
		},
		Client: ClientConfig{
			Token:    "test-token-1",
			Proxy:    true,
			Domains:  []string{"foo.example.org"},
			Interval: Duration(5 * time.Minute),
			Endpoint: "dyndns.example.org",
		},
	}
	c2 := Config{
		LogLevel: "debug",
		Server: ServerConfig{
			Port:    80,
			Domains: []string{"example.com"},
		},
		Client: ClientConfig{
			Token:    "test-token-2",
			Proxy:    false,
			Domains:  []string{"bar.example.net"},
			Interval: Duration(10 * time.Minute),
			Endpoint: "dyndns.example.net",
		},
	}
	ssl := DefaultConfig()
	ssl.Server.Port = 443
	ssl.Server.SSL = SSLConfig{
		Enabled: true,
		Cert:    "server.crt",
		Key:     "server.key",
	}
	tMatrix := []struct {
		Name, Path, Mode string
		Result           Config
	}{
		{
			Name:   "EmptyConfig",
			Path:   "",
			Mode:   MODE_SERVER,
			Result: DefaultConfig(),
		},
		{
			Name:   "ServerConfig1",
			Path:   "testdata/valid-config-1.yaml",
			Mode:   MODE_SERVER,
			Result: c1,
		},
		{
			Name:   "ServerConfig2",
			Path:   "testdata/valid-config-2.yaml",
			Mode:   MODE_SERVER,
			Result: c2,
		},
		{
			Name:   "ClientConfig1",
			Path:   "testdata/valid-config-1.yaml",
			Mode:   MODE_CLIENT,
			Result: c1,
		},
		{
			Name:   "ClientConfig2",
			Path:   "testdata/valid-config-2.yaml",
			Mode:   MODE_CLIENT,
			Result: c2,
		},
		{
			Name:   "RelayConfig1",
			Path:   "testdata/valid-config-1.yaml",
			Mode:   MODE_RELAY,
			Result: c1,
		},
		{
			Name:   "RelayConfig2",
			Path:   "testdata/valid-config-2.yaml",
			Mode:   MODE_RELAY,
			Result: c2,
		},
		{
			Name:   "ServerConfigSSL",
			Path:   "testdata/valid-config-ssl.yaml",
			Mode:   MODE_SERVER,
			Result: ssl,
		},
	}

	for _, tCase := range tMatrix {
		t.Run(tCase.Name, func(t *testing.T) {
			c, err := LoadConfig(tCase.Path, tCase.Mode, false)

			assert := assert.New(t)

			if !assert.Nil(err) {
				t.Fatalf("Failed to load config: %v", err)
			}
			assert.Equal(tCase.Result, c)
		})
	}
}

func TestInvalidConfig(t *testing.T) {
	tMatrix := []struct {
		Name, Path, Mode, Error string
	}{
		{
			Name:  "InvalidPath",
			Path:  "file-does-not-exist.yaml",
			Error: "*fs.PathError",
		},
		{
			Name:  "NotYaml",
			Path:  "testdata/not-a-config.txt",
			Error: "*fmt.wrapError",
		},
		{
			Name:  "ClientMissingToken",
			Mode:  MODE_CLIENT,
			Path:  "testdata/invalid-config-1.yaml",
			Error: "dyndns.ErrMissingToken",
		},
		{
			Name:  "ClientNoDomain",
			Mode:  MODE_CLIENT,
			Path:  "testdata/invalid-config-2.yaml",
			Error: "dyndns.ErrNoDomain",
		},
		{
			Name:  "ClientWrongInterval",
			Mode:  MODE_CLIENT,
			Path:  "testdata/invalid-config-3.yaml",
			Error: "*fmt.wrapError",
		},
		{
			Name:  "ClientInvalidInterval",
			Mode:  MODE_CLIENT,
			Path:  "testdata/invalid-config-5.yaml",
			Error: "*config.ErrInvalidInterval",
		},
		{
			Name:  "RelayMissingToken",
			Mode:  MODE_RELAY,
			Path:  "testdata/invalid-config-1.yaml",
			Error: "dyndns.ErrMissingToken",
		},
		{
			Name:  "RelayNoDomain",
			Mode:  MODE_RELAY,
			Path:  "testdata/invalid-config-2.yaml",
			Error: "dyndns.ErrNoDomain",
		},
		{
			Name:  "RelayWrongInterval",
			Mode:  MODE_RELAY,
			Path:  "testdata/invalid-config-3.yaml",
			Error: "*fmt.wrapError",
		},
		{
			Name:  "RelayMissingEndpoint",
			Mode:  MODE_RELAY,
			Path:  "testdata/invalid-config-4.yaml",
			Error: "dyndns.ErrMissingEndpoint",
		},
		{
			Name:  "RelayInvalidInterval",
			Mode:  MODE_RELAY,
			Path:  "testdata/invalid-config-5.yaml",
			Error: "*config.ErrInvalidInterval",
		},
		{
			Name:  "ServerIncompleteSSLConfig1",
			Mode:  MODE_SERVER,
			Path:  "testdata/invalid-config-ssl-1.yaml",
			Error: "config.ErrIncompleteSSLConfig",
		},
		{
			Name:  "ServerIncompleteSSLConfig2",
			Mode:  MODE_SERVER,
			Path:  "testdata/invalid-config-ssl-2.yaml",
			Error: "config.ErrIncompleteSSLConfig",
		},
	}

	for _, tCase := range tMatrix {
		t.Run(tCase.Name, func(t *testing.T) {
			_, err := LoadConfig(tCase.Path, tCase.Mode, false)

			if !assert.Error(t, err) {
				t.Fatal("Did not receive an error")
			}
			if !assert.Equal(t, tCase.Error, reflect.TypeOf(err).String()) {
				t.Fatalf("Received invalid error: %v", err)
			}
		})
	}
}

func TestEnvSubstitution(t *testing.T) {
	c := Config{
		LogLevel: "debug",
		Server: ServerConfig{
			Port:    2080,
			Domains: []string{"example.org", "example.net"},
		},
		Client: ClientConfig{
			Token:    "token-from-env",
			Proxy:    true,
			Domains:  []string{"foo.example.org"},
			Interval: Duration(15 * time.Minute),
			Endpoint: "dyndns.example.org",
		},
	}
	t.Setenv("DYNDNS_TEST_LOG_LEVEL", c.LogLevel)
	t.Setenv("DYNDNS_TEST_SERVER_PORT", strconv.Itoa(c.Server.Port))
	t.Setenv("DYNDNS_TEST_SERVER_DOMAIN1", c.Server.Domains[0])
	t.Setenv("DYNDNS_TEST_SERVER_DOMAIN2", c.Server.Domains[1])
	t.Setenv("DYNDNS_TEST_CLIENT_TOKEN", c.Client.Token)
	t.Setenv("DYNDNS_TEST_CLIENT_PROXY", strconv.FormatBool(c.Client.Proxy))
	t.Setenv("DYNDNS_TEST_CLIENT_DOMAIN", c.Client.Domains[0])
	t.Setenv("DYNDNS_TEST_CLIENT_INTERVAL", c.Client.Interval.String())
	t.Setenv("DYNDNS_TEST_CLIENT_ENDPOINT", c.Client.Endpoint)

	modes := []string{MODE_SERVER, MODE_CLIENT, MODE_RELAY}

	for _, mode := range modes {
		t.Run(mode, func(t *testing.T) {
			res, err := LoadConfig("testdata/env-config.yaml", mode, true)

			assert := assert.New(t)

			if !assert.Nil(err) {
				t.Fatalf("Could not load config: %v", err)
			}
			assert.Equal(c, res)
		})
	}
}

func TestSetLogLevel(t *testing.T) {
	tMatrix := []struct {
		Name  string
		Level slog.Level
		Error error
	}{
		{"debug", slog.LevelDebug, nil},
		{"info", slog.LevelInfo, nil},
		{"warn", slog.LevelWarn, nil},
		{"error", slog.LevelError, nil},
		{"DEBUG", slog.LevelDebug, nil},
		{"INFO", slog.LevelInfo, nil},
		{"WARN", slog.LevelWarn, nil},
		{"ERROR", slog.LevelError, nil},
		{"Unknown", 0, &ErrUnknownLogLevel{"Unknown"}},
	}
	t.Cleanup(func() {
		err := setLogLevel(DEFAULT_LOG_LEVEL)
		if err != nil {
			t.Fatalf("Failed to cleanup after test: %v", err)
		}
	})

	for _, tCase := range tMatrix {
		t.Run(tCase.Name, func(t *testing.T) {
			err := setLogLevel(tCase.Name)

			assert := assert.New(t)

			if !assert.Equal(tCase.Error, err) {
				t.Fatalf("Received invalid error: %v", err)
			}
			if err == nil {
				assert.Equal(tCase.Level, logLevel.Level())
			}
		})
	}
}
