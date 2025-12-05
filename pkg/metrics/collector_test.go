package metrics

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
)

func TestNewCollector(t *testing.T) {
	assert := assert.New(t)

	c := NewCollector()
	assert.NotNil(c, "Collector instance should not be nil")
	assert.NotNil(c.ipv4Counter, "IPv4 counter should be initialized")
	assert.NotNil(c.ipv6Counter, "IPv6 counter should be initialized")
	assert.NotNil(c.requestCounter, "Request counter should be initialized")
}

func TestCollect(t *testing.T) {
	assert := assert.New(t)

	c := NewCollector()
	ch := make(chan prometheus.Metric)

	c.ipv4Counter.WithLabelValues("example.com").Inc()
	c.ipv6Counter.WithLabelValues("example.com").Inc()
	c.requestCounter.WithLabelValues("GET", "200").Inc()
	go func() {
		c.Collect(ch)
		close(ch)
	}()

	count := 0
	for range ch {
		count++
	}

	assert.Equal(count, 3, "Should collect the exact number of metrics")
}

func TestIncrementIPCounters(t *testing.T) {
	assert := assert.New(t)

	assert.NotPanics(func() {
		ChangedIPv4([]string{"example.com"})
	}, "Incrementing IPv4 counters should not panic")
	assert.NotPanics(func() {
		ChangedIPv6([]string{"example.com"})
	}, "Incrementing IPv6 counters should not panic")

	ms = &metricsServer{
		collector: NewCollector(),
	}
	t.Cleanup(func() {
		ms = nil
	})

	ChangedIPv4([]string{"example.com"})
	ChangedIPv6([]string{"example.com"})

	ch := make(chan prometheus.Metric)

	go func() {
		ms.collector.Collect(ch)
		close(ch)
	}()

	count := 0
	for range ch {
		count++
	}

	assert.Equal(count, 2, "Should collect the exact number of metrics")
}

func TestDomainsToString(t *testing.T) {
	tMatrix := map[string]struct {
		input    []string
		expected string
	}{
		"SingleDomain": {
			input:    []string{"example.com"},
			expected: "example.com",
		},
		"MultipleDomains": {
			input:    []string{"example.com", "test.com", "mydomain.org"},
			expected: "example.com;test.com;mydomain.org",
		},
		"NoDomains": {
			input:    []string{},
			expected: "",
		},
	}

	for name, tc := range tMatrix {
		t.Run(name, func(t *testing.T) {
			assert.Equal(t, tc.expected, domainsToString(tc.input), "Domains to string conversion failed")
		})
	}
}

func TestMetricWrapper(t *testing.T) {
	assert := assert.New(t)

	called := false

	handler := MetricWrapper(http.HandlerFunc(func(_ http.ResponseWriter, _ *http.Request) {
		called = true
	}))
	req, _ := http.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()

	assert.NotPanics(func() {
		handler.ServeHTTP(rr, req)
	}, "MetricWrapper should not panic")

	assert.True(called, "Wrapped handler should be called")
	called = false

	ms = &metricsServer{
		collector: NewCollector(),
	}
	t.Cleanup(func() {
		ms = nil
	})

	rr = httptest.NewRecorder()
	handler.ServeHTTP(rr, req)
	assert.True(called, "Wrapped handler should be called")

	ch := make(chan prometheus.Metric)

	go func() {
		ms.collector.Collect(ch)
		close(ch)
	}()

	count := 0
	for range ch {
		count++
	}

	assert.Equal(count, 1, "Should collect the exact number of metrics")
}
