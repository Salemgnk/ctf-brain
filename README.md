# 🧩 CTF Companion

> **Le carnet du hacker, mais intelligent**

Une application TUI (Terminal User Interface) en Rust pour organiser vos sessions Hack The Box, picoCTF et autres CTF.

## ✨ Fonctionnalités

- 📦 **Gestion de machines CTF** - Enregistrez vos boxes HTB, picoCTF, TryHackMe
- 📝 **Notes catégorisées** - Organisez vos découvertes (Web, Pwn, Crypto, Recon, etc.)
- 🔧 **Tracking d'actions** - Gardez trace de ce que vous avez testé
- 🎨 **Interface TUI moderne** - Navigation au clavier avec ratatui
- 💾 **Persistence locale** - Sauvegarde automatique en JSON

## 🚀 Installation

```bash
git clone git@github.com:Salemgnk/ctf-brain.git
cd ctf-brain
cargo build --release
```

## 📖 Utilisation

```bash
cargo run
```

### Contrôles clavier

| Touche | Action |
|--------|--------|
| `j` / `↓` | Descendre dans la liste |
| `k` / `↑` | Monter dans la liste |
| `Enter` | Voir les détails |
| `Esc` | Retour à la liste |
| `q` | Quitter |

## 🎯 Statut du projet

**Phase 1 - MVP Interface** ✅
- [x] Vue liste des boxes
- [x] Vue détail avec notes et actions
- [x] Navigation clavier
- [ ] Persistence JSON

**Phase 2 - Gestion** 🚧
- [ ] Ajout de boxes
- [ ] Édition de notes
- [ ] Suppression

**Phase 3 - Intelligence** 📋
- [ ] Suggestions de pistes
- [ ] Détection de patterns
- [ ] Export de writeups

## 🛠️ Stack technique

- **Rust** - Langage
- **ratatui** - Interface TUI
- **serde** - Serialization JSON
- **crossterm** - Terminal events

## 📝 License

MIT
