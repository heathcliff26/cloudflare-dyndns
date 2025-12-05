package metrics

import (
	"log/slog"
	"net/http"
	"strconv"
	"strings"

	"github.com/prometheus/client_golang/prometheus"
)

type collector struct {
	ipv4Counter    *prometheus.CounterVec
	ipv6Counter    *prometheus.CounterVec
	requestCounter *prometheus.CounterVec
}

// Create a new collector with the metrics initialized
func NewCollector() *collector {
	return &collector{
		ipv4Counter: prometheus.NewCounterVec(prometheus.CounterOpts{
			Name: "dyndns_changed_ipv4_total",
			Help: "Total number of times the IPv4 address has changed",
		}, []string{"domains"}),
		ipv6Counter: prometheus.NewCounterVec(prometheus.CounterOpts{
			Name: "dyndns_changed_ipv6_total",
			Help: "Total number of times the IPv6 address has changed",
		}, []string{"domains"}),
		requestCounter: prometheus.NewCounterVec(prometheus.CounterOpts{
			Name: "dyndns_requests_total",
			Help: "Total number of requests made to update the DNS records",
		}, []string{"method", "status"}),
	}
}

// Implements the Describe function for prometheus.Collector
func (c *collector) Describe(ch chan<- *prometheus.Desc) {
	c.ipv4Counter.Describe(ch)
	c.ipv6Counter.Describe(ch)
	c.requestCounter.Describe(ch)
}

// Implements the Collect function for prometheus.Collector
func (c *collector) Collect(ch chan<- prometheus.Metric) {
	slog.Debug("Starting collection of metrics for cloudflare-dyndns")
	c.ipv4Counter.Collect(ch)
	c.ipv6Counter.Collect(ch)
	c.requestCounter.Collect(ch)
}

// Increment the IPv4 changed counter for the given domains
func ChangedIPv4(domains []string) {
	if ms == nil {
		return
	}
	ms.collector.ipv4Counter.WithLabelValues(domainsToString(domains)).Inc()
}

// Increment the IPv6 changed counter for the given domains
func ChangedIPv6(domains []string) {
	if ms == nil {
		return
	}
	ms.collector.ipv6Counter.WithLabelValues(domainsToString(domains)).Inc()
}

// Convert a list of domains to a semicolon-separated string
func domainsToString(domains []string) string {
	return strings.Join(domains, ";")
}

// MetricWrapper is an HTTP middleware that wraps the given handler to collect metrics about incoming requests.
func MetricWrapper(next http.Handler) http.Handler {
	return http.HandlerFunc(func(res http.ResponseWriter, req *http.Request) {
		if ms == nil {
			next.ServeHTTP(res, req)
			return
		}

		// Wrap the ResponseWriter to capture the status code
		wrapped := &responseWrapper{
			ResponseWriter: res,
			statusCode:     http.StatusOK,
		}

		next.ServeHTTP(wrapped, req)

		ms.collector.requestCounter.WithLabelValues(req.Method, strconv.Itoa(wrapped.statusCode)).Inc()
	})
}
