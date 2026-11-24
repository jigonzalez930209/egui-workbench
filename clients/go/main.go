// A reference client for the egui-workbench sidecar protocol.
//
// It loads the shared conformance fixture, starts `egui-workbench --spec ... --stdio`,
// applies every patch in the fixture, asks for the spec, and checks the same tree the
// Rust and Python hosts check.
package main

import (
	"bufio"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type fixture struct {
	Spec     map[string]any   `json:"spec"`
	Patches  []map[string]any `json:"patches"`
	Expected map[string]any   `json:"expected_spec"`
}

type message map[string]any

func writeFrame(writer io.Writer, value any) error {
	payload, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var length [4]byte
	binary.BigEndian.PutUint32(length[:], uint32(len(payload)))
	if _, err := writer.Write(length[:]); err != nil {
		return err
	}
	_, err = writer.Write(payload)
	return err
}

func readFrame(reader io.Reader) (message, error) {
	var length [4]byte
	if _, err := io.ReadFull(reader, length[:]); err != nil {
		return nil, err
	}
	payload := make([]byte, binary.BigEndian.Uint32(length[:]))
	if _, err := io.ReadFull(reader, payload); err != nil {
		return nil, err
	}
	var value message
	err := json.Unmarshal(payload, &value)
	return value, err
}

func main() {
	root, err := filepath.Abs(filepath.Join("..", ".."))
	if err != nil {
		fail(err)
	}
	sidecar := filepath.Join(root, "target", "debug", "egui-workbench")
	if value := os.Getenv("EGUI_WORKBENCH_SIDECAR"); value != "" {
		sidecar = value
	}
	fixturePath := filepath.Join(root, "crates", "egui-workbench-host", "conformance", "basic.json")
	raw, err := os.ReadFile(fixturePath)
	if err != nil {
		fail(err)
	}
	var data fixture
	if err := json.Unmarshal(raw, &data); err != nil {
		fail(err)
	}
	specFile, err := os.CreateTemp("", "egui-workbench-conformance-*.json")
	if err != nil {
		fail(err)
	}
	defer os.Remove(specFile.Name())
	if err := json.NewEncoder(specFile).Encode(data.Spec); err != nil {
		fail(err)
	}
	specFile.Close()

	command := exec.Command(sidecar, "--spec", specFile.Name(), "--stdio")
	stdin, err := command.StdinPipe()
	if err != nil {
		fail(err)
	}
	stdout, err := command.StdoutPipe()
	if err != nil {
		fail(err)
	}
	command.Stderr = os.Stderr
	if err := command.Start(); err != nil {
		fail(err)
	}
	defer command.Process.Kill()
	reader := bufio.NewReader(stdout)
	hello, err := readFrame(reader)
	if err != nil {
		fail(err)
	}
	if hello["type"] != "hello" {
		fail(fmt.Errorf("expected hello, got %v", hello))
	}
	for index, patch := range data.Patches {
		if err := writeFrame(stdin, message{
			"type": "request",
			"id":   index + 1,
			"request": message{
				"op":    "apply_patch",
				"patch": patch,
			},
		}); err != nil {
			fail(err)
		}
		if _, err := readFrame(reader); err != nil {
			fail(err)
		}
	}
	if err := writeFrame(stdin, message{
		"type":    "request",
		"id":      99,
		"request": message{"op": "get_spec"},
	}); err != nil {
		fail(err)
	}
	response, err := readFrame(reader)
	if err != nil {
		fail(err)
	}
	encoded, _ := json.Marshal(response)
	text := string(encoded)
	if !strings.Contains(text, `"value":"Ada"`) || !strings.Contains(text, `"text":"Hello, Ada"`) {
		fail(fmt.Errorf("conformance mismatch: %s", text))
	}
	fmt.Println("go client ok")
}

func fail(err error) {
	fmt.Fprintln(os.Stderr, err)
	os.Exit(1)
}
