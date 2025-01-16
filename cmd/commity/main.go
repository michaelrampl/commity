package main

import (
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/michaelrampl/commity/internal/config"
	"github.com/michaelrampl/commity/internal/utils"

	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
)

var VERSION = "MASTER"

var style_error = lipgloss.NewStyle().Foreground(lipgloss.Color("1"))
var style_warning = lipgloss.NewStyle().Foreground(lipgloss.Color("3"))
var style_success = lipgloss.NewStyle().Foreground(lipgloss.Color("#233ee7"))

type ParamMap map[string]string

func (m *ParamMap) String() string {
	return fmt.Sprintf("%v", *m)
}

func (m *ParamMap) Set(value string) error {
	parts := strings.Split(value, "=")
	if len(parts) != 2 {
		return fmt.Errorf("invalid map entry: %s", value)
	}
	(*m)[parts[0]] = parts[1]
	return nil
}

func getTheme() *huh.Theme {
	var t *huh.Theme = huh.ThemeBase()

	// Colors
	var colorPrimary = lipgloss.AdaptiveColor{
		Light: "#161616",
		Dark:  "#f1f1f1",
	}
	var colorSecondary = lipgloss.AdaptiveColor{
		Light: "#686868",
		Dark:  "#a4a4a4",
	}
	var colorHighlight = lipgloss.AdaptiveColor{
		Light: "#233ee7",
		Dark:  "#237ce7",
	}

	// Basic
	t.Focused.Title = t.Focused.Title.Foreground(colorPrimary).Bold(true).PaddingTop(0)
	t.Focused.Description = t.Focused.Description.Foreground(colorSecondary).PaddingBottom(0)
	t.Focused.Base = lipgloss.NewStyle().PaddingLeft(0).BorderStyle(lipgloss.HiddenBorder()).BorderLeft(false)

	// Error Indicator
	t.Focused.ErrorIndicator = lipgloss.NewStyle().SetString(" *")
	t.Focused.ErrorMessage = lipgloss.NewStyle().SetString(" *")

	// Select
	t.Focused.SelectSelector = t.Focused.SelectSelector.SetString("")
	t.Focused.SelectedOption = t.Focused.SelectedOption.SetString("● ").Foreground(colorHighlight)
	t.Focused.UnselectedOption = t.Focused.UnselectedOption.SetString("○ ")

	// TextInput
	t.Focused.TextInput.Text = t.Focused.TextInput.Text.Foreground(colorPrimary)
	t.Focused.TextInput.Prompt = lipgloss.NewStyle().Foreground(colorPrimary)

	// Button
	t.Focused.FocusedButton = t.Focused.FocusedButton.Background(colorHighlight)
	t.Focused.BlurredButton = t.Focused.BlurredButton.Background(colorSecondary)

	// Footer
	t.Help.Ellipsis = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.FullKey = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.FullDesc = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.FullSeparator = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.ShortKey = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.ShortDesc = lipgloss.NewStyle().Foreground(colorSecondary)
	t.Help.ShortSeparator = lipgloss.NewStyle().Foreground(colorSecondary)

	return t

}

func runCommity(directory string, paramMap ParamMap) {

	repoPath, err := utils.FindGitRepository(directory)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error findig git repository: %v", err)))
		os.Exit(1)
	}

	hasStagedFiles, err := utils.HasStagedChanges(directory)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error checking added files: %v", err)))
		os.Exit(1)
	}
	if !hasStagedFiles {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Nothing to commit in %v", repoPath)))
		os.Exit(1)
	}

	// Load the configuration file
	cfg, err := utils.LoadConfig(repoPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error loading configuration: %v", err)))
		os.Exit(1)
	}

	if len(cfg.Entries) == 0 || cfg.Template == "" {
		fmt.Fprintln(os.Stderr, style_error.Render("Invalid configuration: no entries or template provided"))
		os.Exit(1)
	}

	var groups []*huh.Group

	for _, entry := range cfg.Entries {
		switch e := entry.(type) {
		case *config.TextEntry:
			if paramMap[e.Name] != "" {
				e.Value = paramMap[e.Name]
			}
			if e.MultiLine {
				group := huh.NewGroup(huh.NewText().
					Value(&e.Value).
					Title(e.Label).
					Description(e.Description),
				)
				groups = append(groups, group)
			} else {
				group := huh.NewGroup(huh.NewInput().
					Value(&e.Value).
					Title(e.Label).
					Description(e.Description),
				)
				groups = append(groups, group)
			}

		case *config.ChoiceEntry:
			var options []huh.Option[string]
			for _, choice := range e.Choices {
				options = append(options, huh.NewOption(choice.Label, choice.Value))
				if paramMap[e.Name] != "" && paramMap[e.Name] == choice.Value {
					e.Value = paramMap[e.Name]
				}
			}

			group := huh.NewGroup(huh.NewSelect[string]().
				Value(&e.Value).
				Title(e.Label).
				Description(e.Description).
				Options(options...),
			)
			groups = append(groups, group)
		case *config.BooleanEntry:
			if paramMap[e.Name] != "" {
				if strings.ToLower(paramMap[e.Name]) == "true" || paramMap[e.Name] == "1" {
					e.Value = true
				} else {
					e.Value = false
				}
			}
			group := huh.NewGroup(huh.NewConfirm().
				Value(&e.Value).
				Title(e.Label).
				Description(e.Description),
			)
			groups = append(groups, group)
		default:
			fmt.Fprintln(os.Stderr, style_error.Render("Unknown entry type"))
			os.Exit(1)
		}

	}

	form := huh.NewForm(groups...).WithTheme(getTheme())
	err = form.Run()
	if err != nil {
		if err == huh.ErrUserAborted { // Check if the user canceled the form
			fmt.Println(style_warning.Render("Commit Canceled - Goodbye!"))
			os.Exit(1)
		}
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error running commity: %v", err)))
		os.Exit(1)
	}

	msg, err := utils.RenderCommitMessage(cfg)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error rendering commit message: %v", err)))
		os.Exit(1)
	}

	err = utils.Commit(repoPath, msg)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error while doing commit: %v", err)))
		os.Exit(1)
	}

	fmt.Println(style_success.Render("Commit successful!"))
	fmt.Println(msg)
}

func main() {

	// Define cmdline parameters
	version := flag.Bool("version", false, "Print the version and exit")
	help := flag.Bool("help", false, "Show help message and exit")
	directory := flag.String("directory", "", "The directory to run commity in")
	paramMap := ParamMap{}
	flag.Var(&paramMap, "map", "Set default values for the form (e.g., -map key1=value1 -map key2=value2)")

	// Parse the flags
	flag.Parse()

	// Handle version and help flags
	if *version {
		fmt.Printf("Version: %s\n", VERSION)
		return
	}
	if *help {
		fmt.Println("Usage: commity [options]")
		fmt.Println("Options:")
		flag.PrintDefaults()
		return
	}

	// handle the project directory
	var dir = *directory
	var err error
	if dir == "" {
		dir, err = os.Getwd()
		if err != nil {
			fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error getting current directory: %v", err)))
			os.Exit(1)
		}
	}
	runCommity(dir, paramMap)

}
