package utils

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"text/template"

	"github.com/michaelrampl/commity/internal/config"

	"github.com/go-git/go-git/v5"
	gitcfg "github.com/go-git/go-git/v5/config"
)

// FindGitRepository searches for the nearest Git repository starting from the given directory.
// It traverses upward in the directory hierarchy until it finds a `.git` directory or reaches the root.
//
// Arguments:
// - startDir: The starting directory to search for the Git repository.
//
// Returns:
// - The path to the nearest Git repository, or an error if none is found.
func FindGitRepository(startDir string) (string, error) {
	dir := startDir

	for {
		// Check if the .git directory exists
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir, nil
		}

		// Use go-git to check if this is a valid Git repository
		_, err := git.PlainOpen(dir)
		if err == nil {
			return dir, nil
		}

		// Move up one directory
		parentDir := filepath.Dir(dir)
		if parentDir == dir {
			break // reached the root directory
		}
		dir = parentDir
	}

	return "", fmt.Errorf("no Git repository found")
}

// GetDataDir retrieves the application data directory path.
// It constructs the path to the user's local configuration directory and appends the application-specific subdirectory.
//
// Returns:
// - The path to the application data directory, or an error if the user's config directory cannot be determined.
func GetDataDir() (string, error) {
	// Use os.UserConfigDir for the user's local config directory
	dataDir, err := os.UserConfigDir()
	if err != nil || dataDir == "" {
		return "", errors.New("unable to determine the data directory")
	}

	// Append the application-specific directory
	appDataDir := filepath.Join(dataDir, "commity")
	return appDataDir, nil
}

// LoadConfig locates and loads the configuration file.
// It checks each directory from repoPath up to root for a `.commity.yaml`.
// If none is found, it loads the global config from the user data dir.
//
// Arguments:
// - repoPath: The starting directory to search for the repo-specific config.
//
// Returns:
// - cfg:       The Configuration loaded from the found file.
// - configPath: The full path to the config file that was loaded.
// - error:     If no config file is found or parsing fails.
func LoadConfig(repoPath string) (*config.Configuration, string, error) {
	// 1) Try walking up from repoPath
	dir := repoPath
	for {
		candidate := filepath.Join(dir, ".commity.yaml")
		if _, err := os.Stat(candidate); err == nil {
			// found a repo-local config
			cfg, err := config.ParseConfigFile(candidate)
			return cfg, candidate, err
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break // reached filesystem root
		}
		dir = parent
	}

	// 2) Fallback to global config in your data dir
	dataDir, err := GetDataDir()
	if err != nil {
		return nil, "", err
	}
	globalConfig := filepath.Join(dataDir, "commity.yaml")
	if _, err := os.Stat(globalConfig); err == nil {
		cfg, err := config.ParseConfigFile(globalConfig)
		return cfg, globalConfig, err
	}

	return nil, "", fmt.Errorf(
		"no config file found in any parent of %s or in %s",
		repoPath, globalConfig,
	)
}

// RenderCommitMessage generates a commit message using the template string in the configuration.
// It populates the template with field names and their corresponding values from the configuration entries.
//
// Arguments:
// - config: A pointer to the Configuration struct containing the template and entries.
//
// Returns:
// - The rendered commit message as a string, or an error if the rendering process fails.
func RenderCommitMessage(config *config.Configuration) (string, error) {
	if config.Template == "" {
		return "", fmt.Errorf("template string is empty")
	}

	// Prepare a map holding the data for the template
	vars := make(map[string]interface{})
	for _, entry := range config.Entries {
		vars[entry.GetName()] = entry.GetValue()
	}

	// Parse the template string and execute the template engine
	tmpl, err := template.New("message").Parse(config.Template)
	if err != nil {
		return "", fmt.Errorf("failed to parse template: %w", err)
	}

	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, vars); err != nil {
		return "", fmt.Errorf("failed to execute template: %w", err)
	}

	return buf.String(), nil
}

// Commit creates a new commit in the specified Git repository.
//
// Arguments:
// - repoPath: The path to the Git repository where the commit should be created.
// - message: The commit message.
//
// Returns:
// - An error if the commit cannot be created.
func Commit(repoPath string, message string) error {
	// Open the Git repository.
	repo, err := git.PlainOpen(repoPath)
	if err != nil {
		return fmt.Errorf("failed to open Git repository: %w", err)
	}

	// Get the working tree of the repository.
	worktree, err := repo.Worktree()
	if err != nil {
		return fmt.Errorf("failed to get Git worktree: %w", err)
	}

	// Create the commit with the provided message.
	_, err = worktree.Commit(message, &git.CommitOptions{})
	if err != nil {
		return fmt.Errorf("failed to create commit: %w", err)
	}

	return nil
}

// GetStagedFiles checks if there are any staged changes in the specified Git repository.
//
// Arguments:
// - repoPath: The path to the Git repository.
//
// Returns:
// - The number of staged files
// - An error if the repository cannot be accessed or status cannot be retrieved.
func GetStagedFiles(repoPath string) (int, error) {
	// Open the Git repository.
	repo, err := git.PlainOpen(repoPath)
	if err != nil {
		return 0, fmt.Errorf("failed to open Git repository: %w", err)
	}

	// Get the working tree.
	worktree, err := repo.Worktree()
	if err != nil {
		return 0, fmt.Errorf("failed to get Git worktree: %w", err)
	}

	// Get the status of the working tree.
	status, err := worktree.Status()
	if err != nil {
		return 0, fmt.Errorf("failed to get Git status: %w", err)
	}

	// Check if any file has a staged change.
	stagedFiles := 0
	for _, fileStatus := range status {
		if fileStatus.Staging != git.Unmodified && fileStatus.Staging != git.Untracked {
			stagedFiles++
		}
	}

	return stagedFiles, nil
}

// GetGitIdentity retrieves the Git identity (user.name and user.email)
// by checking, in order, the local repo config, your global config, and then system config.
// It returns the first scope in which both name and email are set.
func GetGitIdentity(repoPath string) (name string, email string, err error) {
	// Open the repository
	repo, err := git.PlainOpen(repoPath)
	if err != nil {
		return "", "", fmt.Errorf("failed to open Git repository: %w", err)
	}

	// Try each scope in precedence order
	scopes := []gitcfg.Scope{
		gitcfg.LocalScope,
		gitcfg.GlobalScope,
		gitcfg.SystemScope,
	}
	for _, scope := range scopes {
		cfg, cfgErr := repo.ConfigScoped(scope)
		if cfgErr != nil {
			// e.g. no such file for system scope
			continue
		}
		if cfg.User.Name != "" && cfg.User.Email != "" {
			return cfg.User.Name, cfg.User.Email, nil
		}
	}

	return "", "", fmt.Errorf("git user.name and/or user.email not set in any config scope")
}
