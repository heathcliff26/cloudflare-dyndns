package metrics

import (
	"fmt"
	"math/rand/v2"
	"net/http"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInitMetricsAndServe(t *testing.T) {
	t.Run("MetricsDisabled", func(t *testing.T) {
		assert := assert.New(t)

		InitMetricsAndServe(MetricsOptions{})
		assert.Nil(ms, "Metrics server instance should be nil when disabled")
	})

	tMatrix := map[string]MetricsOptions{
		"AllEnabled": {
			GoCollector:      true,
			ProcessCollector: true,
		},
		"GoOnly": {
			GoCollector:      true,
			ProcessCollector: false,
		},
		"ProcessOnly": {
			GoCollector:      false,
			ProcessCollector: true,
		},
		"NoneEnabled": {
			GoCollector:      false,
			ProcessCollector: false,
		},
	}

	port := rand.IntN(65535-10000) + 10000 // Random port between 10000 and 65534

	for name, opts := range tMatrix {
		t.Run(name, func(t *testing.T) {
			opts.Enabled = true
			opts.Port = port
			InitMetricsAndServe(opts)

			assert := assert.New(t)

			t.Cleanup(func() {
				assert.NoError(Close(), "Should close metrics server gracefully")
			})

			assert.NotNil(ms, "Metrics server instance should not be nil")

			require.Eventually(t, func() bool {
				req, _ := http.NewRequest(http.MethodGet, fmt.Sprintf("http://localhost:%d/metrics", opts.Port), nil)
				req.Header.Add("Accept", "text/plain")
				res, err := http.DefaultClient.Do(req)
				if err != nil {
					t.Logf("Failed to reach metrics server: %v", err)
					return false
				}
				defer res.Body.Close()

				if res.StatusCode != http.StatusOK {
					t.Logf("Metrics server returned non-200 status: %d", res.StatusCode)
					return false
				}

				return true
			}, time.Second*30, time.Millisecond*500, "Metrics server should be reachable")

			assert.NotNil(ms.server, "Should have http server instance")

			metrics, err := ms.registry.Gather()
			assert.NoError(err, "Gathering metrics should not error")

			var GoCollectorFound, ProcessCollectorFound bool
			for _, m := range metrics {
				if m.GetName() == "go_goroutines" {
					GoCollectorFound = true
				}
				if m.GetName() == "process_cpu_seconds_total" {
					ProcessCollectorFound = true
				}
			}

			assert.Equal(opts.GoCollector, GoCollectorFound, "GoCollector presence should match configuration")
			assert.Equal(opts.ProcessCollector, ProcessCollectorFound, "ProcessCollector presence should match configuration")
		})
	}
}

func TestClose(t *testing.T) {
	assert := assert.New(t)

	// Closing when ms is nil should be no-op
	ms = nil
	assert.NoError(Close(), "Closing nil metrics server should not error")

	// Closing when no server is running should just set ms to nil
	ms = &metricsServer{}
	assert.NoError(Close(), "Closing metrics server with no running server should not error")
	assert.Nil(ms, "Metrics server instance should be nil after close")
}
