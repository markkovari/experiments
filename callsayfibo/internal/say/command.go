package say

import "os/exec"

func Say(params ...string) error {
	result := exec.Command("say", params...)
	return result.Run()
}
