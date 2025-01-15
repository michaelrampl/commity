package utils

import (
	"bytes"
	"commity/internal/config"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"text/template"

	"github.com/go-git/go-git/v5"
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

// getDataDir retrieves the application data directory path.
// It constructs the path to the user's local configuration directory and appends the application-specific subdirectory.
//
// Returns:
// - The path to the application data directory, or an error if the user's config directory cannot be determined.
func getDataDir() (string, error) {
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
// It checks both the repository-specific and global locations for the configuration file.
//
// Arguments:
// - directory: The starting directory to search for the configuration file.
//
// Returns:
// - A pointer to the Configuration struct loaded from the file, or an error if the file cannot be found or loaded.
func LoadConfig(repoPath string) (*config.Configuration, error) {

	dataDir, err := getDataDir()
	if err != nil {
		return nil, err
	}

	configLocal := filepath.Join(repoPath, ".commity.yaml")
	configGlobal := filepath.Join(dataDir, "commity.yaml")

	var configPath string
	if _, err := os.Stat(configLocal); err == nil {
		configPath = configLocal
	} else if _, err := os.Stat(configGlobal); err == nil {
		configPath = configGlobal
	} else {
		return nil, fmt.Errorf("no config file found in %s or %s", configLocal, configGlobal)
	}

	return config.ParseConfigFile(configPath)
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

// GetAddedFiles returns a list of files that are marked as added in the Git repository.
//
// Arguments:
// - repoPath: The path to the Git repository.
//
// Returns:
// - A slice of strings representing the added files.
// - An error if the repository cannot be accessed or status cannot be retrieved.
func GetAddedFiles(repoPath string) (int, error) {
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

	// Collect files with added status.
	var addedFiles = 0
	for _, fileStatus := range status {
		// Check if the file is added to the staging area or the working tree.
		if fileStatus.Staging == git.Added || fileStatus.Worktree == git.Added {
			addedFiles += 1
		}
	}

	return addedFiles, nil
}
