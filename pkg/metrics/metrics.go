package metrics

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/collectors"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

const DefaultMetricServerPort = 9090

// The internal instance for metrics server
var ms *metricsServer

type metricsServer struct {
	port      int
	registry  *prometheus.Registry
	server    *http.Server
	collector *collector
}

type MetricsOptions struct {
	// Enable determines if metrics should be collected and served
	Enabled bool `json:"enabled,omitempty"`
	// The port to serve metrics on. Metrics will be served on /metrics endpoint
	Port int `json:"port,omitempty"`
	// Enable Go runtime metrics
	GoCollector bool `json:"goCollector,omitempty"`
	// Enable process metrics
	ProcessCollector bool `json:"processCollector,omitempty"`
}

func DefaultMetricsOptions() MetricsOptions {
	return MetricsOptions{
		Enabled:          false,
		Port:             DefaultMetricServerPort,
		GoCollector:      true,
		ProcessCollector: true,
	}
}

// Initialize the metrics package to enable collecting and serving metrics.
// Will call os.Exit(1) if the server fails to start.
func InitMetricsAndServe(opts MetricsOptions) {
	if !opts.Enabled {
		slog.Debug("Metrics are disabled")
		return
	}

	reg := prometheus.NewRegistry()

	c := NewCollector()
	reg.MustRegister(c)

	if opts.GoCollector {
		slog.Debug("Enabling Go runtime metrics collector")
		reg.MustRegister(collectors.NewGoCollector())
	}

	if opts.ProcessCollector {
		slog.Debug("Enabling process metrics collector")
		reg.MustRegister(collectors.NewProcessCollector(collectors.ProcessCollectorOpts{}))
	}

	ms = &metricsServer{
		port:      opts.Port,
		registry:  reg,
		collector: c,
	}

	go serve()
}

func serve() {
	ms.server = &http.Server{
		Addr:         fmt.Sprintf(":%d", ms.port),
		Handler:      promhttp.HandlerFor(ms.registry, promhttp.HandlerOpts{Registry: ms.registry}),
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
	}

	slog.Info("Starting metrics server", slog.String("addr", ms.server.Addr))
	err := ms.server.ListenAndServe()
	if err != nil && !errors.Is(err, http.ErrServerClosed) {
		slog.Error("Failed to start metrics server", "err", err)
		os.Exit(1)
	}
}

// Shutdown metric server gracefully and set the instance to nil
func Close() error {
	if ms == nil {
		return nil
	}
	if ms.server == nil {
		ms = nil
		return nil
	}
	defer func() { ms = nil }()
	return ms.server.Shutdown(context.Background())
}
