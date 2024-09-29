package cloudflaredyndns

import (
	"testing"

	"github.com/heathcliff26/cloudflare-dyndns/pkg/version"
	"github.com/stretchr/testify/assert"
)

func TestNewRootCommand(t *testing.T) {
	cmd := NewRootCommand()

	assert.Equal(t, version.Name, cmd.Use)
}
