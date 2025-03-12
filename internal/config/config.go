package config

import (
	"fmt"
	"os"
	"strings"

	"gopkg.in/yaml.v3"
)

// Entry defines common behavior for all entry types.
// Each entry type must implement the GetName and GetValue methods.
type Entry interface {
	GetName() string
	GetValue() interface{}
}

// TextEntry represents a text input field in the configuration.
// It supports optional constraints like minimum and maximum length and whether it allows multiple lines.
type TextEntry struct {
	Name        string `yaml:"name"`        // The unique name of the entry
	Label       string `yaml:"label"`       // A user-friendly label for the entry
	Description string `yaml:"description"` // A description of the entry
	MinLength   int    `yaml:"minLength"`   // Minimum length of the text
	MaxLength   int    `yaml:"maxLength"`   // Maximum length of the text
	MultiLine   bool   `yaml:"multiLine"`   // Whether the text entry supports multiple lines
	Pattern     string `yaml:"pattern"`     // A regular expression pattern to validate the text
	PatternHint string `yaml:"patternHint"` // A hint to display when the pattern does not match
	Default     string `yaml:"default"`     // Default value for the entry
	Value       string `yaml:"-"`           // Runtime value (not serialized to YAML)
	Store       bool   `yaml:"store"`       // Whether to store the for the next run
}

// GetName returns the name of the text entry.
func (e *TextEntry) GetName() string {
	return e.Name
}

// GetValue returns the runtime value of the text entry.
func (e *TextEntry) GetValue() interface{} {
	return e.Value
}

// ChoiceEntry represents a choice input field in the configuration.
// It allows selecting one value from a predefined list of choices.
type ChoiceEntry struct {
	Name        string   `yaml:"name"`        // The unique name of the entry
	Label       string   `yaml:"label"`       // A user-friendly label for the entry
	Description string   `yaml:"description"` // A description of the entry
	Choices     []Choice `yaml:"choices"`     // Available choices for the entry
	Default     string   `yaml:"default"`     // Default selected choice
	Value       string   `yaml:"-"`           // Runtime value (not serialized to YAML)
	Store       bool     `yaml:"store"`       // Whether to store the for the next run
	ShowValues  bool     `yaml:"showValues"`  // Whether to show the internal values of the choices
}

// GetName returns the name of the choice entry.
func (e *ChoiceEntry) GetName() string {
	return e.Name
}

// GetValue returns the runtime value of the choice entry.
func (e *ChoiceEntry) GetValue() interface{} {
	return e.Value
}

// BooleanEntry represents a boolean input field in the configuration.
// It allows toggling a true/false value.
type BooleanEntry struct {
	Name        string `yaml:"name"`        // The unique name of the entry
	Label       string `yaml:"label"`       // A user-friendly label for the entry
	Description string `yaml:"description"` // A description of the entry
	Default     bool   `yaml:"default"`     // Default value for the entry
	Value       bool   `yaml:"-"`           // Runtime value (not serialized to YAML)
	Store       bool   `yaml:"store"`       // Whether to store the for the next run
}

// GetName returns the name of the boolean entry.
func (e *BooleanEntry) GetName() string {
	return e.Name
}

// GetValue returns the runtime value of the boolean entry.
func (e *BooleanEntry) GetValue() interface{} {
	return e.Value
}

// Choice represents a single selectable option for a ChoiceEntry.
type Choice struct {
	Value string `yaml:"value"` // The internal value of the choice
	Label string `yaml:"label"` // The display label for the choice
}

// Configuration holds all the configuration entries and the template string for rendering outputs.
type Configuration struct {
	Entries  []Entry `yaml:"entries"`  // A list of entries in the configuration
	Template string  `yaml:"template"` // A template string for rendering outputs
	Overview bool    `yaml:"overview"` // Whether to show an overview at the beginning of the form
}

// UnmarshalYAML handles the deserialization of the Configuration structure.
// It dynamically parses entries based on their type field and populates the Entries slice.
func (c *Configuration) UnmarshalYAML(value *yaml.Node) error {
	// Define a temporary struct to parse the YAML
	var raw struct {
		Entries  []yaml.Node `yaml:"entries"`
		Template string      `yaml:"template"`
		Overview bool        `yaml:"overview"`
	}
	if err := value.Decode(&raw); err != nil {
		return err
	}

	c.Template = raw.Template
	c.Overview = raw.Overview

	// Parse each entry dynamically based on its type field
	for _, node := range raw.Entries {
		var entryType struct {
			Type string `yaml:"type"`
		}
		if err := node.Decode(&entryType); err != nil {
			return err
		}

		var entry Entry
		switch entryType.Type {
		case "Text":
			var textEntry TextEntry
			if err := node.Decode(&textEntry); err != nil {
				return err
			}
			textEntry.Value = textEntry.Default
			entry = &textEntry
		case "Choice":
			var choiceEntry ChoiceEntry
			if err := node.Decode(&choiceEntry); err != nil {
				return err
			}
			if choiceEntry.ShowValues {
				maxValueLength := 0
				for _, choice := range choiceEntry.Choices {
					if len(choice.Value) > maxValueLength {
						maxValueLength = len(choice.Value)
					}
				}
				for i, choice := range choiceEntry.Choices {
					choiceEntry.Choices[i].Label = fmt.Sprintf("%s%s %s", choice.Value, strings.Repeat(" ", maxValueLength-len(choice.Value)), choice.Label)
				}
			}

			choiceEntry.Value = choiceEntry.Default
			entry = &choiceEntry
		case "Boolean":
			var booleanEntry BooleanEntry
			if err := node.Decode(&booleanEntry); err != nil {
				return err
			}
			booleanEntry.Value = booleanEntry.Default
			entry = &booleanEntry
		default:
			return fmt.Errorf("unknown entry type: %s", entryType.Type)
		}

		c.Entries = append(c.Entries, entry)
	}

	return nil
}

// ParseConfigFile reads a YAML configuration file from the specified path and parses it into a Configuration struct.
//
// Arguments:
// - path: The file path to the YAML configuration file.
//
// Returns:
// - A pointer to the Configuration struct if the file is successfully parsed.
// - An error if the file cannot be read or if parsing fails.
func ParseConfigFile(path string) (*Configuration, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var config Configuration
	decoder := yaml.NewDecoder(file)
	if err := decoder.Decode(&config); err != nil {
		return nil, err
	}

	return &config, nil
}
