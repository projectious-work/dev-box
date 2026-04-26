"""State machine loading and transition validation.

State machines live in two places:

- ``src/primitives/state-machines/<name>.yaml`` — defaults shipped by processkit
- ``context/state-machines/<name>.yaml`` — project overrides

The override is preferred when both exist. Each file is itself a
processkit entity (``kind: StateMachine``) with ``spec.initial``,
``spec.terminal``, and ``spec.states``.
"""
from __future__ import annotations

from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Any

import yaml

from . import paths


class StateMachineError(ValueError):
    """Raised when a state machine is missing or a transition is invalid."""


@dataclass
class StateMachine:
    name: str
    initial: str
    terminal: list[str]
    states: dict[str, dict[str, Any]]

    def is_terminal(self, state: str) -> bool:
        return state in self.terminal

    def known_states(self) -> list[str]:
        return list(self.states.keys())

    def transitions_from(self, state: str) -> list[str]:
        if state not in self.states:
            return []
        out = []
        for t in self.states[state].get("transitions", []) or []:
            if isinstance(t, dict) and "to" in t:
                out.append(t["to"])
        return out

    def can_transition(self, from_state: str, to_state: str) -> bool:
        return to_state in self.transitions_from(from_state)


@lru_cache(maxsize=16)
def load(name: str, sm_dir: Path | None = None) -> StateMachine:
    """Load the state machine ``name``.

    Looks in (in order): consumer override directory, processkit defaults.
    """
    sm_dir = sm_dir or paths.state_machines_dir()
    if sm_dir is None:
        raise StateMachineError("no state-machines directory found")
    candidate = sm_dir / f"{name}.yaml"
    if not candidate.is_file():
        raise StateMachineError(f"no state machine {name!r} at {candidate}")
    text = candidate.read_text()
    try:
        data = yaml.safe_load(text)
    except yaml.YAMLError as e:
        raise StateMachineError(f"invalid YAML in {candidate}: {e}") from e
    if not isinstance(data, dict) or data.get("kind") != "StateMachine":
        raise StateMachineError(f"{candidate} is not a StateMachine entity")
    spec = data.get("spec", {})
    if "initial" not in spec or "states" not in spec:
        raise StateMachineError(f"{candidate} missing spec.initial or spec.states")
    return StateMachine(
        name=name,
        initial=spec["initial"],
        terminal=list(spec.get("terminal", [])),
        states=dict(spec["states"]),
    )


def validate_transition(machine_name: str, from_state: str, to_state: str) -> None:
    """Raise StateMachineError if the transition is not allowed."""
    machine = load(machine_name)
    if from_state not in machine.states:
        raise StateMachineError(
            f"unknown current state {from_state!r} for machine {machine_name!r} "
            f"(known: {sorted(machine.known_states())})"
        )
    if to_state not in machine.states:
        raise StateMachineError(
            f"unknown target state {to_state!r} for machine {machine_name!r}"
        )
    if not machine.can_transition(from_state, to_state):
        allowed = machine.transitions_from(from_state)
        raise StateMachineError(
            f"transition {from_state!r} → {to_state!r} not allowed in {machine_name!r}; "
            f"allowed from {from_state!r}: {allowed or 'none (terminal)'}"
        )
