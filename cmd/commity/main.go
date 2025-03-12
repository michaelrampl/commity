package main

import (
	"crypto/md5"
	"errors"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/michaelrampl/commity/internal/config"
	"github.com/michaelrampl/commity/internal/utils"
	"gopkg.in/yaml.v3"

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

// getStoredKeys returns a dictionary mapping each configuration entry name
// to its store flag (true if the entry should be persisted).
func getStoredKeys(entries *[]config.Entry) map[string]bool {
	storeDict := make(map[string]bool)
	for _, entry := range *entries {
		switch e := entry.(type) {
		case *config.TextEntry:
			storeDict[e.Name] = e.Store
		case *config.ChoiceEntry:
			storeDict[e.Name] = e.Store
		case *config.BooleanEntry:
			storeDict[e.Name] = e.Store
		}
	}
	return storeDict
}

// getCacheFilePath returns the repository-specific cache file path.
// It places the file inside a "cache" subfolder within the data directory
// and uses the MD5 hash of the repository path (in hexadecimal) as the file name.
func getCacheFilePath(repoPath string) (string, error) {
	dataDir, err := utils.GetDataDir()
	if err != nil {
		return "", err
	}
	// Define the cache subfolder.
	cacheDir := filepath.Join(dataDir, "cache")
	// Compute the MD5 hash of repoPath inline.
	hash := md5.Sum([]byte(repoPath))
	repoID := fmt.Sprintf("%x", hash)
	fileName := repoID + ".yaml"
	return filepath.Join(cacheDir, fileName), nil
}

// loadParamMapFromFile loads the persisted parameters from the repository-specific cache file.
func loadParamMapFromFile(repoPath string) (ParamMap, error) {
	cacheFile, err := getCacheFilePath(repoPath)
	if err != nil {
		return nil, err
	}
	data, err := os.ReadFile(cacheFile)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			// No persisted file yet; return an empty map.
			return ParamMap{}, nil
		}
		return nil, err
	}
	var pm ParamMap
	if err := yaml.Unmarshal(data, &pm); err != nil {
		return nil, err
	}
	return pm, nil
}

// saveParamMapToFile persists the parameter map as YAML into the repository-specific cache file.
// It ensures that the cache directory is created if it doesn't exist.
func saveParamMapToFile(paramMap ParamMap, repoPath string) error {
	cacheFile, err := getCacheFilePath(repoPath)
	if err != nil {
		return err
	}
	// Ensure the cache directory exists.
	cacheDir := filepath.Dir(cacheFile)
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		return err
	}
	data, err := yaml.Marshal(paramMap)
	if err != nil {
		return err
	}
	return os.WriteFile(cacheFile, data, 0644)
}

// restoreParamMap merges file-loaded parameters into the provided paramMap,
// but only for keys that are marked as stored (i.e. storedKeys[key] is true)
// and only when the key is not already set (so that CLI–provided values always win).
func restoreParamMap(paramMap *ParamMap, storedKeys map[string]bool, repoPath string) {
	// Load persisted parameters from file.
	fileParams, err := loadParamMapFromFile(repoPath)
	if err != nil {
		// If loading fails, proceed with an empty map.
		fileParams = ParamMap{}
	}
	// For each key marked for storage...
	for key, store := range storedKeys {
		if store {
			// Only merge if the key is not already set in the in-memory map.
			if _, exists := (*paramMap)[key]; !exists {
				if val, ok := fileParams[key]; ok {
					(*paramMap)[key] = val
				}
			}
		}
	}
}

// updateParamMap creates a new parameter map from the configuration entries (cfg.Entries)
// but only for keys marked for storage in storedKeys, and then saves that map to file.
func updateParamMap(entries *[]config.Entry, storedKeys map[string]bool, repoPath string) {
	newMap := make(ParamMap)
	for _, entry := range *entries {
		name := entry.GetName()
		if stored, ok := storedKeys[name]; ok && stored {
			switch e := entry.(type) {
			case *config.TextEntry:
				newMap[name] = e.Value
			case *config.ChoiceEntry:
				newMap[name] = e.Value
			case *config.BooleanEntry:
				if e.Value {
					newMap[name] = "true"
				} else {
					newMap[name] = "false"
				}
			}
		}
	}
	// Persist the new map.
	if err := saveParamMapToFile(newMap, repoPath); err != nil {
		fmt.Fprintln(os.Stderr, style_warning.Render(fmt.Sprintf("Warning: failed to save parameter map: %v", err)))
	}
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

	t.Focused.NoteTitle = lipgloss.NewStyle().PaddingLeft(0).BorderStyle(lipgloss.HiddenBorder()).BorderLeft(false).Foreground(colorPrimary).PaddingBottom(1)

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

// validateInput checks that the value meets the minimum and maximum length requirements.
// Addtionally, a regex pattern may be provided which will be evaluated if not empty
func validateInput(value string, minLength, maxLength int, pattern string, patternHint string) error {
	if len(value) < minLength {
		return fmt.Errorf("Input must be at least %d characters (got %d)", minLength, len(value))
	}
	if maxLength > 0 && len(value) > maxLength {
		return fmt.Errorf("Input must be at most %d characters (got %d)", maxLength, len(value))
	}
	if pattern != "" {
		re, err := regexp.Compile(pattern)
		if err != nil {
			return fmt.Errorf("invalid pattern: %v", err)
		}
		if !re.MatchString(value) {
			if patternHint == "" {
				return fmt.Errorf("input does not match required pattern: %s", pattern)
			}
			return fmt.Errorf("input does not match required pattern: %s", patternHint)

		}
	}
	return nil
}

func runCommity(directory string, paramMap ParamMap) {

	repoPath, err := utils.FindGitRepository(directory)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error findig git repository: %v", err)))
		os.Exit(1)
	}

	stagedFiles, err := utils.GetStagedFiles(repoPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("Error checking added files: %v", err)))
		os.Exit(1)
	}
	if stagedFiles == 0 {
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

	storedKeys := getStoredKeys(&cfg.Entries)

	if len(storedKeys) > 0 {
		restoreParamMap(&paramMap, storedKeys, repoPath)
	}

	var groups []*huh.Group

	if cfg.Overview {
		groups = append(groups, huh.NewGroup(huh.NewNote().
			Title(fmt.Sprintf("Commiting to %s", repoPath)).Description(fmt.Sprintf("You have %d file(s) staged for commit", stagedFiles)),
		))
	}

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
					Description(e.Description).
					Validate(func(input string) error {
						return validateInput(input, e.MinLength, e.MaxLength, e.Pattern, e.PatternHint)
					}),
				)
				groups = append(groups, group)
			} else {
				group := huh.NewGroup(huh.NewInput().
					Value(&e.Value).
					Title(e.Label).
					Description(e.Description).
					Validate(func(input string) error {
						return validateInput(input, e.MinLength, e.MaxLength, e.Pattern, e.PatternHint)
					}),
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

	if len(storedKeys) > 0 {
		updateParamMap(&cfg.Entries, storedKeys, repoPath)
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
	abs_dir, err := filepath.Abs(dir)
	if err != nil {
		fmt.Fprintln(os.Stderr, style_error.Render(fmt.Sprintf("failed to get absolute path: %v", err)))
		os.Exit(1)
	}
	runCommity(abs_dir, paramMap)

}
