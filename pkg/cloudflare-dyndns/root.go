package cloudflaredyndns

import (
	"fmt"
	"os"

	"github.com/heathcliff26/cloudflare-dyndns/pkg/client"
	"github.com/heathcliff26/cloudflare-dyndns/pkg/relay"
	"github.com/heathcliff26/cloudflare-dyndns/pkg/server"
	"github.com/heathcliff26/cloudflare-dyndns/pkg/version"
	"github.com/spf13/cobra"
)

func NewRootCommand() *cobra.Command {
	cobra.AddTemplateFunc(
		"ProgramName", func() string {
			return version.Name
		},
	)

	rootCmd := &cobra.Command{
		Use:   version.Name,
		Short: version.Name + " provides DynDNS functionality for cloudflare.",
		RunE: func(cmd *cobra.Command, _ []string) error {
			return cmd.Help()
		},
	}

	serverCMD, err := server.NewCommand()
	if err != nil {
		exitError(rootCmd, err)
	}

	clientCMD, err := client.NewCommand()
	if err != nil {
		exitError(rootCmd, err)
	}

	relayCMD, err := relay.NewCommand()
	if err != nil {
		exitError(rootCmd, err)
	}

	rootCmd.AddCommand(
		serverCMD,
		clientCMD,
		relayCMD,
		version.NewCommand(),
	)

	return rootCmd
}

func Execute() {
	cmd := NewRootCommand()
	err := cmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

// Print the error information on stderr and exit with code 1
func exitError(cmd *cobra.Command, err error) {
	fmt.Fprintln(cmd.Root().ErrOrStderr(), "Fatal: "+err.Error())
	os.Exit(1)
}
