//! The command registry: registration, bindings, and shortcut polling.

use std::collections::{BTreeMap, VecDeque};

use egui::Context;

use super::{BindingError, Command, CommandId, KeyChord, KeySequence, WhenContext};

/// A default binding that was skipped because another command already owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingConflict {
    /// The command whose default was ignored.
    pub command: CommandId,
    /// The command that kept the sequence.
    pub existing: CommandId,
    /// The sequence.
    pub sequence: KeySequence,
}

/// The command registry.
#[derive(Clone, Debug, Default)]
pub struct CommandRegistry {
    commands: BTreeMap<CommandId, Command>,
    overrides: BTreeMap<CommandId, Option<KeySequence>>,
    conflicts: Vec<BindingConflict>,
    recent: VecDeque<CommandId>,
    pending: Vec<KeyChord>,
    pending_at: f64,
}

impl CommandRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command. A default shortcut that conflicts with an earlier command is
    /// skipped and recorded in [`Self::conflicts`].
    pub fn register(&mut self, command: Command) -> Option<Command> {
        let id = command.id.clone();
        if let Some(sequence) = command.default_shortcut.clone()
            && let Some(existing) = self.owner_of(&sequence, Some(&id))
        {
            self.conflicts.push(BindingConflict {
                command: id.clone(),
                existing,
                sequence,
            });
        }
        self.commands.insert(id, command)
    }

    /// Register several commands.
    pub fn register_all(&mut self, commands: impl IntoIterator<Item = Command>) {
        for command in commands {
            self.register(command);
        }
    }

    /// Remove a command and its binding.
    pub fn remove(&mut self, id: &CommandId) -> Option<Command> {
        self.overrides.remove(id);
        self.recent.retain(|recent| recent != id);
        self.commands.remove(id)
    }

    /// A command by id.
    pub fn get(&self, id: &CommandId) -> Option<&Command> {
        self.commands.get(id)
    }

    /// A command by id, mutably.
    pub fn get_mut(&mut self, id: &CommandId) -> Option<&mut Command> {
        self.commands.get_mut(id)
    }

    /// Every command, sorted by id.
    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.values()
    }

    /// The number of commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether no command is registered.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Set the enabled state.
    pub fn set_enabled(&mut self, id: &CommandId, enabled: bool) -> bool {
        match self.commands.get_mut(id) {
            Some(command) => {
                command.state.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, id: &CommandId, checked: bool) -> bool {
        match self.commands.get_mut(id) {
            Some(command) => {
                command.state.checked = checked;
                true
            }
            None => false,
        }
    }

    /// The palette rows for the commands that apply in `context`.
    pub fn palette_items(&self, context: &WhenContext) -> Vec<crate::PaletteItem<CommandId>> {
        super::palette::command_palette_items(self, context)
    }

    /// The categories in display order.
    pub fn categories(&self) -> Vec<&str> {
        let mut categories: Vec<&str> = self
            .commands
            .values()
            .map(|command| command.category.as_str())
            .collect();
        categories.sort_unstable();
        categories.dedup();
        categories
    }

    /// The default shortcut of a command.
    pub fn default_binding(&self, id: &CommandId) -> Option<&KeySequence> {
        self.commands
            .get(id)
            .and_then(|command| command.default_shortcut.as_ref())
    }

    /// The active shortcut of a command, including overrides.
    pub fn binding(&self, id: &CommandId) -> Option<&KeySequence> {
        match self.overrides.get(id) {
            Some(override_binding) => override_binding.as_ref(),
            None => self.default_binding(id),
        }
    }

    /// Bind a command to a sequence, replacing its default. Fails when another command
    /// owns the sequence.
    ///
    /// # Errors
    /// When the command is unknown or another command already owns the sequence.
    pub fn bind(&mut self, id: &CommandId, sequence: KeySequence) -> Result<(), BindingError> {
        if !self.commands.contains_key(id) {
            return Err(BindingError::UnknownCommand(id.clone()));
        }
        if let Some(existing) = self.owner_of(&sequence, Some(id)) {
            return Err(BindingError::Conflict { sequence, existing });
        }
        self.overrides.insert(id.clone(), Some(sequence));
        Ok(())
    }

    /// Remove the active binding of a command.
    pub fn unbind(&mut self, id: &CommandId) {
        self.overrides.insert(id.clone(), None);
    }

    /// Return a command to its default binding.
    pub fn reset_binding(&mut self, id: &CommandId) {
        self.overrides.remove(id);
    }

    /// Whether the command has a user override (including an explicit unbind).
    pub fn has_override(&self, id: &CommandId) -> bool {
        self.overrides.contains_key(id)
    }

    /// The command that owns a sequence, if any.
    pub fn owner_of(&self, sequence: &KeySequence, except: Option<&CommandId>) -> Option<CommandId> {
        self.commands
            .iter()
            .filter(|(id, _)| except.is_none_or(|except| *id != except))
            .find(|(id, _command)| self.binding(id).is_some_and(|binding| binding == sequence))
            .map(|(id, _)| id.clone())
    }

    /// Defaults that were ignored because another command owned the sequence.
    pub fn conflicts(&self) -> &[BindingConflict] {
        &self.conflicts
    }

    /// Every active binding, longest sequence first.
    pub fn bindings(&self) -> Vec<(KeySequence, CommandId)> {
        let mut bindings: Vec<(KeySequence, CommandId)> = self
            .commands
            .keys()
            .filter_map(|id| self.binding(id).map(|sequence| (sequence.clone(), id.clone())))
            .collect();
        bindings.sort_by(|left, right| right.0.len().cmp(&left.0.len()).then_with(|| left.1.cmp(&right.1)));
        bindings
    }

    /// The shortcut pressed this frame, if a command applies in `context`.
    ///
    /// Longest sequences are tried first. A sequence with more chords starts a pending
    /// sequence; the next chord completes it within 1.5 s. The key is consumed so text
    /// fields and widgets do not see it.
    pub fn shortcut(&mut self, ctx: &Context, context: &WhenContext) -> Option<CommandId> {
        let now = ctx.input(|input| input.time);
        if !self.pending.is_empty() && now - self.pending_at > 1.5 {
            self.pending.clear();
        }
        let candidates: Vec<(KeySequence, CommandId)> = self
            .bindings()
            .into_iter()
            .filter(|(_, id)| {
                self.commands
                    .get(id)
                    .is_some_and(|command| command.is_enabled() && command.applies(context))
            })
            .collect();

        if self.pending.is_empty() {
            for (sequence, id) in &candidates {
                let Some(first) = sequence.chords().first() else {
                    continue;
                };
                if ctx.input_mut(|input| input.consume_key(first.modifiers, first.key)) {
                    if sequence.len() == 1 {
                        self.note_used(id.clone());
                        return Some(id.clone());
                    }
                    self.pending.push(*first);
                    self.pending_at = now;
                    return None;
                }
            }
            return None;
        }

        let pending = self.pending.clone();
        let mut matched = false;
        for (sequence, id) in &candidates {
            if !sequence.starts_with(&pending) {
                continue;
            }
            if sequence.len() <= pending.len() {
                continue;
            }
            let chord = sequence.chords()[pending.len()];
            if ctx.input_mut(|input| input.consume_key(chord.modifiers, chord.key)) {
                self.pending.push(chord);
                self.pending_at = now;
                if self.pending.len() == sequence.len() {
                    self.pending.clear();
                    self.note_used(id.clone());
                    return Some(id.clone());
                }
                matched = true;
                break;
            }
        }
        if !matched {
            self.pending.clear();
        }
        None
    }

    /// Mark a command as recently used, for palette ranking.
    pub fn note_used(&mut self, id: CommandId) {
        self.recent.retain(|recent| recent != &id);
        self.recent.push_front(id);
        while self.recent.len() > 12 {
            self.recent.pop_back();
        }
    }

    /// Recently used commands, newest first.
    pub fn recent(&self) -> impl Iterator<Item = &CommandId> {
        self.recent.iter()
    }

    /// Clear the recent list.
    pub fn clear_recent(&mut self) {
        self.recent.clear();
    }

    /// The chord currently being waited for, if a sequence has started.
    pub fn pending(&self) -> &[KeyChord] {
        &self.pending
    }

    /// A key press from a UI input, for the keybindings editor.
    pub fn pressed_chord(ctx: &Context) -> Option<KeyChord> {
        let mut chord = None;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    chord = Some(KeyChord::new(*modifiers, *key));
                    break;
                }
            }
        });
        chord
    }
}

impl FromIterator<Command> for CommandRegistry {
    fn from_iter<T: IntoIterator<Item = Command>>(iter: T) -> Self {
        let mut registry = Self::new();
        registry.register_all(iter);
        registry
    }
}

#[cfg(test)]
mod tests;
