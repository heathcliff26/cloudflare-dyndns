package server

import (
	"log/slog"
	"os"

	"github.com/heathcliff26/cloudflare-dyndns/pkg/config"
	"github.com/spf13/cobra"
)

const (
	flagNameConfig = "config"
	flagNameEnv    = "env"
)

func NewCommand() (*cobra.Command, error) {
	cmd := &cobra.Command{
		Use:   config.MODE_SERVER,
		Short: "Run a server for relay clients",
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

	cmd.Flags().Bool(flagNameEnv, false, "Expand enviroment variables in config file")

	return cmd, nil
}

func run(configPath string, env bool) {
	cfg, err := config.LoadConfig(configPath, config.MODE_SERVER, env)
	if err != nil {
		slog.Error("Could not load configuration", slog.String("path", configPath), slog.String("err", err.Error()))
		os.Exit(1)
	}

	s := NewServer(cfg.Server)
	err = s.Run()
	if err != nil {
		slog.Error("Failed to start the server", "err", err)
		os.Exit(1)
	}
}
