package relay

import (
	"log/slog"
	"os"
	"time"

	"github.com/heathcliff26/cloudflare-dyndns/pkg/config"
	"github.com/heathcliff26/cloudflare-dyndns/pkg/dyndns"
	"github.com/spf13/cobra"
)

const (
	flagNameConfig = "config"
	flagNameEnv    = "env"
)

func NewCommand() (*cobra.Command, error) {
	cmd := &cobra.Command{
		Use:   config.MODE_RELAY,
		Short: "Update DDNS Records but relay the calls through a server",
		RunE: func(cmd *cobra.Command, _ []string) error {
			cfg, err := cmd.Flags().GetString(flagNameConfig)
			if err != nil {
				return err
			}

			env, err := cmd.Flags().GetBool(flagNameEnv)
			if err != nil {
				return err
			}

			run(cfg, env)
			return nil
		},
	}

	cmd.Flags().StringP(flagNameConfig, "c", "", "Path to config file")
	err := cmd.MarkFlagFilename(flagNameConfig, "yaml", "yml")
	if err != nil {
		return nil, err
	}
	err = cmd.MarkFlagRequired(flagNameConfig)
	if err != nil {
		return nil, err
	}

	cmd.Flags().Bool(flagNameEnv, false, "Expand enviroment variables in config file")

	return cmd, nil
}

func run(configPath string, env bool) {
	cfg, err := config.LoadConfig(configPath, config.MODE_RELAY, env)
	if err != nil {
		slog.Error("Could not load configuration", slog.String("path", configPath), slog.String("err", err.Error()))
		os.Exit(1)
	}

	r, err := NewRelay(cfg.Client.Token, cfg.Client.Proxy, cfg.Client.Endpoint)
	if err != nil {
		slog.Error("Could not create new client", "err", err)
		os.Exit(1)
	}
	r.Data().SetDomains(cfg.Client.Domains)
	dyndns.Run(r, time.Duration(cfg.Client.Interval))
}
