// Example: register, activate, serve, and ping using wispers-connect (Go).
//
// Compatible with wconnect, the Python example, and the C example.
// Uses the same wire protocol:
//   - QUIC: send "PING\n", expect "PONG\n"
//   - UDP:  send "ping",   expect "pong"
//
// Usage:
//
//	wconnect-go [--hub ADDR] [--storage DIR] status
//	wconnect-go [--hub ADDR] [--storage DIR] register TOKEN
//	wconnect-go [--hub ADDR] [--storage DIR] activate CODE
//	wconnect-go [--hub ADDR] [--storage DIR] nodes
//	wconnect-go [--hub ADDR] [--storage DIR] serve
//	wconnect-go [--hub ADDR] [--storage DIR] ping [--quic] NODE_NUM
package main

import (
	"fmt"
	"os"
	"os/signal"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	wispersgo "dev.wispers.dev/connect"
)

// --- FileStorage: StorageCallbacks backed by files on disk ---

type FileStorage struct {
	dir string
}

func NewFileStorage(dir string) (*FileStorage, error) {
	if err := os.MkdirAll(dir, 0700); err != nil {
		return nil, fmt.Errorf("create storage dir: %w", err)
	}
	return &FileStorage{dir: dir}, nil
}

func (fs *FileStorage) LoadRootKey() ([]byte, error) {
	data, err := os.ReadFile(filepath.Join(fs.dir, "root_key.bin"))
	if os.IsNotExist(err) {
		return nil, nil // not found
	}
	return data, err
}

func (fs *FileStorage) SaveRootKey(key []byte) error {
	return os.WriteFile(filepath.Join(fs.dir, "root_key.bin"), key, 0600)
}

func (fs *FileStorage) DeleteRootKey() error {
	err := os.Remove(filepath.Join(fs.dir, "root_key.bin"))
	if os.IsNotExist(err) {
		return nil
	}
	return err
}

func (fs *FileStorage) LoadRegistration() ([]byte, error) {
	data, err := os.ReadFile(filepath.Join(fs.dir, "registration.pb"))
	if os.IsNotExist(err) {
		return nil, nil // not found
	}
	return data, err
}

func (fs *FileStorage) SaveRegistration(data []byte) error {
	return os.WriteFile(filepath.Join(fs.dir, "registration.pb"), data, 0600)
}

func (fs *FileStorage) DeleteRegistration() error {
	err := os.Remove(filepath.Join(fs.dir, "registration.pb"))
	if os.IsNotExist(err) {
		return nil
	}
	return err
}

// --- Default storage path (matches wconnect --profile default) ---

func defaultStorageDir() string {
	switch runtime.GOOS {
	case "darwin":
		home, _ := os.UserHomeDir()
		return filepath.Join(home, "Library", "Application Support", "wconnect", "default")
	default:
		if xdg := os.Getenv("XDG_CONFIG_HOME"); xdg != "" {
			return filepath.Join(xdg, "wconnect", "default")
		}
		home, _ := os.UserHomeDir()
		return filepath.Join(home, ".config", "wconnect", "default")
	}
}

// --- CLI parsing ---

type cliArgs struct {
	hub     string
	storage string
	command string
	args    []string // positional args after command
	quic    bool     // --quic flag for ping
}

func parseArgs() (*cliArgs, error) {
	args := os.Args[1:]
	cli := &cliArgs{}

	i := 0
	// Parse global flags
	for i < len(args) && strings.HasPrefix(args[i], "--") {
		switch {
		case args[i] == "--hub" && i+1 < len(args):
			cli.hub = args[i+1]
			i += 2
		case strings.HasPrefix(args[i], "--hub="):
			cli.hub = args[i][6:]
			i++
		case args[i] == "--storage" && i+1 < len(args):
			cli.storage = args[i+1]
			i += 2
		case strings.HasPrefix(args[i], "--storage="):
			cli.storage = args[i][10:]
			i++
		default:
			return nil, fmt.Errorf("unknown flag: %s", args[i])
		}
	}

	if i >= len(args) {
		return nil, fmt.Errorf("no command specified")
	}

	cli.command = args[i]
	i++

	// Parse command-specific flags and args
	for i < len(args) {
		if args[i] == "--quic" {
			cli.quic = true
		} else {
			cli.args = append(cli.args, args[i])
		}
		i++
	}

	return cli, nil
}

func printUsage() {
	fmt.Fprintf(os.Stderr, `Usage:
  wconnect-go [--hub ADDR] [--storage DIR] status
  wconnect-go [--hub ADDR] [--storage DIR] register TOKEN
  wconnect-go [--hub ADDR] [--storage DIR] activate CODE
  wconnect-go [--hub ADDR] [--storage DIR] nodes
  wconnect-go [--hub ADDR] [--storage DIR] serve
  wconnect-go [--hub ADDR] [--storage DIR] ping [--quic] NODE_NUM
`)
}

// --- Helper: init storage + node ---

func initNode(cli *cliArgs) (*wispersgo.NodeStorage, *wispersgo.Node, wispersgo.NodeState, error) {
	dir := cli.storage
	if dir == "" {
		dir = defaultStorageDir()
	}

	fs, err := NewFileStorage(dir)
	if err != nil {
		return nil, nil, 0, err
	}

	storage := wispersgo.NewNodeStorage(fs)
	if cli.hub != "" {
		if err := storage.OverrideHubAddr(cli.hub); err != nil {
			storage.Close()
			return nil, nil, 0, fmt.Errorf("override hub addr: %w", err)
		}
	}

	node, state, err := storage.RestoreOrInit()
	if err != nil {
		storage.Close()
		return nil, nil, 0, fmt.Errorf("restore or init: %w", err)
	}

	return storage, node, state, nil
}

func printGroup(node *wispersgo.Node) {
	info, err := node.GroupInfo()
	if err != nil {
		fmt.Fprintf(os.Stderr, "  Failed to get group info: %v\n", err)
		return
	}
	fmt.Printf("  Group state: %s\n", info.State)
	for _, n := range info.Nodes {
		tag := ""
		if n.IsSelf {
			tag = " (self)"
		}
		online := ""
		if n.IsOnline {
			online = " [online]"
		}
		name := n.Name
		if name == "" {
			name = "(unnamed)"
		}
		status := "Unknown"
		switch n.ActivationStatus {
		case wispersgo.ActivationNotActivated:
			status = "NOT_ACTIVATED"
		case wispersgo.ActivationActivated:
			status = "ACTIVATED"
		}
		fmt.Printf("  Node %d: %s — %s%s%s\n", n.NodeNumber, name, status, tag, online)
	}
}

// --- Commands ---

func cmdStatus(cli *cliArgs) int {
	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	fmt.Printf("Node state: %s\n", state)

	if state != wispersgo.NodeStatePending {
		reg, err := storage.ReadRegistration()
		if err == nil {
			fmt.Printf("Node number: %d, group: %s\n", reg.NodeNumber, reg.ConnectivityGroupID)
		}
		printGroup(node)
	}
	return 0
}

func cmdRegister(cli *cliArgs) int {
	if len(cli.args) < 1 {
		fmt.Fprintf(os.Stderr, "Error: register requires a token\n")
		return 1
	}
	token := cli.args[0]

	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	if state != wispersgo.NodeStatePending {
		fmt.Fprintf(os.Stderr, "Cannot register: already %s\n", state)
		return 1
	}

	fmt.Println("Registering...")
	if err := node.Register(token); err != nil {
		fmt.Fprintf(os.Stderr, "Registration failed: %v\n", err)
		return 1
	}

	fmt.Printf("Registered! State: %s\n", node.State())

	reg, err := storage.ReadRegistration()
	if err == nil {
		fmt.Printf("Node number: %d, group: %s\n", reg.NodeNumber, reg.ConnectivityGroupID)
	}
	return 0
}

func cmdActivate(cli *cliArgs) int {
	if len(cli.args) < 1 {
		fmt.Fprintf(os.Stderr, "Error: activate requires an activation code\n")
		return 1
	}
	code := cli.args[0]

	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	if state == wispersgo.NodeStatePending {
		fmt.Fprintln(os.Stderr, "Cannot activate: not registered yet")
		return 1
	}
	if state == wispersgo.NodeStateActivated {
		fmt.Fprintln(os.Stderr, "Already activated")
		return 1
	}

	fmt.Printf("Activating with code: %s\n", code)
	if err := node.Activate(code); err != nil {
		fmt.Fprintf(os.Stderr, "Activation failed: %v\n", err)
		return 1
	}

	fmt.Printf("Activated! State: %s\n", node.State())
	return 0
}

func cmdNodes(cli *cliArgs) int {
	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	if state == wispersgo.NodeStatePending {
		fmt.Fprintln(os.Stderr, "Not registered yet")
		return 1
	}

	printGroup(node)
	return 0
}

func handleQuicStream(conn *wispersgo.QuicConnection, stream *wispersgo.QuicStream) {
	defer stream.Close()
	defer conn.Close()

	data, err := stream.Read(1024)
	if err != nil {
		fmt.Printf("  Stream read error: %v\n", err)
		return
	}

	line := string(data)
	if idx := strings.Index(line, "\n"); idx >= 0 {
		line = line[:idx]
	}

	if line == "PING" {
		fmt.Println("  Received PING, sending PONG")
		if err := stream.Write([]byte("PONG\n")); err != nil {
			fmt.Printf("  Write error: %v\n", err)
			return
		}
		stream.Finish()
	} else {
		fmt.Printf("  Unknown command: %q\n", line)
	}
}

func handleUdpConnection(conn *wispersgo.UdpConnection) {
	defer conn.Close()
	for {
		data, err := conn.Recv()
		if err != nil {
			fmt.Printf("  UDP connection closed: %v\n", err)
			return
		}
		if string(data) == "ping" {
			fmt.Println("  Received ping, sending pong")
			if err := conn.Send([]byte("pong")); err != nil {
				fmt.Printf("  UDP send error: %v\n", err)
				return
			}
		} else {
			fmt.Printf("  Received %d bytes\n", len(data))
		}
	}
}

func cmdServe(cli *cliArgs) int {
	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	if state == wispersgo.NodeStatePending {
		fmt.Fprintln(os.Stderr, "Cannot serve: not registered yet")
		return 1
	}

	reg, err := storage.ReadRegistration()
	if err == nil {
		fmt.Printf("Node %d in group %s\n", reg.NodeNumber, reg.ConnectivityGroupID)
	}
	fmt.Printf("Starting serving session (state: %s)...\n", state)

	session, err := node.StartServing()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to start serving: %v\n", err)
		return 1
	}
	defer session.Close()

	// Auto-print activation code if group state allows endorsing
	info, err := node.GroupInfo()
	if err == nil {
		if info.State == wispersgo.GroupStateCanEndorse || info.State == wispersgo.GroupStateBootstrap {
			code, err := session.GenerateActivationCode()
			if err == nil {
				fmt.Printf("\nActivation code for a new peer:\n  %s\n\n", code)
			}
		}
	}

	// Run session in background goroutine
	sessionDone := make(chan error, 1)
	go func() {
		sessionDone <- session.Run()
	}()

	// Accept incoming connections
	if session.Incoming != nil {
		fmt.Println("Listening for incoming connections...")
		go func() {
			for {
				conn, err := session.Incoming.AcceptQuic()
				if err != nil {
					return
				}
				fmt.Println("Incoming QUIC connection accepted")
				go func() {
					stream, err := conn.AcceptStream()
					if err != nil {
						fmt.Printf("  Accept stream error: %v\n", err)
						conn.Close()
						return
					}
					handleQuicStream(conn, stream)
				}()
			}
		}()
		go func() {
			for {
				conn, err := session.Incoming.AcceptUdp()
				if err != nil {
					return
				}
				fmt.Println("Incoming UDP connection accepted")
				go handleUdpConnection(conn)
			}
		}()
	}

	fmt.Println("Serving (Ctrl-C to quit)...")

	// Wait for Ctrl-C or session end
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	select {
	case <-sigCh:
		fmt.Println("\nShutting down...")
		session.Shutdown()
	case err := <-sessionDone:
		if err != nil {
			fmt.Printf("Serving session ended: %v\n", err)
		}
	}

	return 0
}

func cmdPing(cli *cliArgs) int {
	if len(cli.args) < 1 {
		fmt.Fprintf(os.Stderr, "Error: ping requires a node number\n")
		return 1
	}
	peer, err := strconv.Atoi(cli.args[0])
	if err != nil || peer <= 0 {
		fmt.Fprintf(os.Stderr, "Error: invalid node number\n")
		return 1
	}

	storage, node, state, err := initNode(cli)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		return 1
	}
	defer storage.Close()
	defer node.Close()

	if state != wispersgo.NodeStateActivated {
		fmt.Fprintf(os.Stderr, "Cannot ping: must be ACTIVATED (currently %s)\n", state)
		return 1
	}

	// Start serving session (needed for P2P)
	session, err := node.StartServing()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to start serving: %v\n", err)
		return 1
	}
	defer session.Close()

	var sessionWg sync.WaitGroup
	sessionWg.Add(1)
	go func() {
		defer sessionWg.Done()
		session.Run()
	}()

	// Accept incoming connections in background
	if session.Incoming != nil {
		go func() {
			for {
				conn, err := session.Incoming.AcceptQuic()
				if err != nil {
					return
				}
				go func() {
					stream, err := conn.AcceptStream()
					if err != nil {
						conn.Close()
						return
					}
					handleQuicStream(conn, stream)
				}()
			}
		}()
		go func() {
			for {
				conn, err := session.Incoming.AcceptUdp()
				if err != nil {
					return
				}
				go handleUdpConnection(conn)
			}
		}()
	}

	transport := "UDP"
	if cli.quic {
		transport = "QUIC"
	}
	fmt.Printf("Pinging node %d via %s...\n", peer, transport)

	start := time.Now()

	if cli.quic {
		conn, err := node.ConnectQuic(int32(peer))
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to connect: %v\n", err)
			return 1
		}
		connectTime := time.Since(start)
		fmt.Printf("  Connected in %.3fs\n", connectTime.Seconds())

		stream, err := conn.OpenStream()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to open stream: %v\n", err)
			conn.Close()
			return 1
		}

		if err := stream.Write([]byte("PING\n")); err != nil {
			fmt.Fprintf(os.Stderr, "Failed to write: %v\n", err)
			stream.Close()
			conn.Close()
			return 1
		}
		stream.Finish()

		pongStart := time.Now()
		reply, err := stream.Read(1024)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to read: %v\n", err)
			stream.Close()
			conn.Close()
			return 1
		}
		rtt := time.Since(pongStart)

		if string(reply) == "PONG\n" {
			fmt.Printf("  Pong received in %.3fs\n", rtt.Seconds())
		} else {
			fmt.Printf("  Unexpected response: %q\n", reply)
		}

		stream.Close()
		conn.Close()
	} else {
		conn, err := node.ConnectUdp(int32(peer))
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to connect: %v\n", err)
			return 1
		}
		connectTime := time.Since(start)
		fmt.Printf("  Connected in %.3fs\n", connectTime.Seconds())

		if err := conn.Send([]byte("ping")); err != nil {
			fmt.Fprintf(os.Stderr, "Failed to send: %v\n", err)
			conn.Close()
			return 1
		}

		pongStart := time.Now()
		reply, err := conn.Recv()
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to recv: %v\n", err)
			conn.Close()
			return 1
		}
		rtt := time.Since(pongStart)

		if string(reply) == "pong" {
			fmt.Printf("  Pong received in %.3fs\n", rtt.Seconds())
		} else {
			fmt.Printf("  Unexpected response: %q\n", reply)
		}

		conn.Close()
	}

	fmt.Printf("Ping successful! Total time: %.3fs\n", time.Since(start).Seconds())

	session.Shutdown()
	return 0
}

func main() {
	cli, err := parseArgs()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		printUsage()
		os.Exit(1)
	}

	var code int
	switch cli.command {
	case "status":
		code = cmdStatus(cli)
	case "register":
		code = cmdRegister(cli)
	case "activate":
		code = cmdActivate(cli)
	case "nodes":
		code = cmdNodes(cli)
	case "serve":
		code = cmdServe(cli)
	case "ping":
		code = cmdPing(cli)
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s\n", cli.command)
		printUsage()
		code = 1
	}
	os.Exit(code)
}
