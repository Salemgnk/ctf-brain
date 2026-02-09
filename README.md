# 🧩 CTF Companion

> **Le carnet du hacker, mais intelligent**

Une application TUI (Terminal User Interface) en Rust pour organiser vos sessions Hack The Box, picoCTF et autres CTF.

## ✨ Fonctionnalités

- 📦 **Gestion de machines CTF** - Enregistrez vos boxes HTB, picoCTF, TryHackMe
- 📝 **Notes catégorisées** - Organisez vos découvertes (Web, Pwn, Crypto, Recon, etc.)
- 🔧 **Tracking d'actions** - Gardez trace de ce que vous avez testé
- 🔐 **Variables d'environnement** - Stockez tokens JWT, cookies, API keys par box
- 🚀 **Lancement de shell** - Shell automatique avec IP et variables chargées
- 📊 **Logging transparent** - Toutes les commandes sont enregistrées
- 🎨 **Interface TUI moderne** - Navigation au clavier avec ratatui
- 💾 **Persistence locale** - Sauvegarde automatique en JSON

## 🚀 Installation

```bash
git clone git@github.com:Salemgnk/ctf-brain.git
cd ctf-brain
cargo build --release
```

## 📖 Utilisation

### Lancement

```bash
cargo run
```

Au premier lancement, ctf-brain :
- Crée `~/.ctf-brain/`
- Installe le hook de logging
- Affiche des données de sample

### Workflow typique

```bash
# 1. Lancer ctf-brain
$ ctf-brain

# 2. Ajouter une box (touche 'a')
#    Titre: Lame
#    Platform: HTB
#    IP: 10.10.10.3
#    Tags: easy, linux

# 3. Voir les détails (Enter)

# 4. Ajouter des variables (touche 'e')
#    JWT_TOKEN = eyJhbGci...
#    API_KEY = sk-proj-...

# 5. Lancer le shell (touche 'l')
[Lame] $ n                    # Alias: nmap -sV $CTF_IP
[Lame] $ echo $JWT_TOKEN      # Variable disponible

# 6. Split ton terminal manuellement

# 7. Dans le nouveau pane:
$ ctf-brain
# Sélectionne "Lame" → 'l'
[Lame] $ g /admin             # Alias: gobuster
```

### Contrôles clavier

#### Vue Liste
| Touche | Action |
|--------|--------|
| `j` / `↓` | Descendre dans la liste |
| `k` / `↑` | Monter dans la liste |
| `Enter` | Voir les détails |
| `a` | Ajouter une box |
| `d` | Supprimer la box sélectionnée |
| `l` | Lancer shell avec environnement |
| `q` | Quitter |

#### Vue Détails
| Touche | Action |
|--------|--------|
| `e` | Éditer les variables d'environnement |
| `l` | Lancer shell |
| `Esc` | Retour à la liste |

#### Édition Variables
| Touche | Action |
|--------|--------|
| `a` | Ajouter une variable |
| `Tab` | Champ suivant |
| `Enter` | Valider |
| `Esc` | Retour |

## 🏗️ Architecture

```
~/.ctf-brain/
├── boxes.json              # Base de données
├── boxes/
│   ├── box-1.env          # Env shell pour chaque box
│   └── box-2.env
├── logs/
│   └── box-1.jsonl        # Logs de commandes
└── shell-hook.sh          # Hook de logging
```

## 🎯 Aliases disponibles dans le shell

Lorsque tu lances un shell avec `l`, tu as accès à :

```bash
ip         # Affiche $CTF_IP
n          # nmap -sV $CTF_IP
na         # nmap -sC -sV -A $CTF_IP
g /path    # gobuster dir -u http://$CTF_IP -w wordlist
nc-listen  # rlwrap nc -lvnp 4444
```

## 🛠️ Stack technique

- **Rust** - Langage
- **ratatui** - Interface TUI
- **serde** - Serialization JSON
- **crossterm** - Terminal events
- **chrono** - Gestion des dates

## 🐛 Troubleshooting

### Le shell ne se lance pas
```bash
# Vérifier que bash est installé
which bash

# Vérifier les permissions
ls -la ~/.ctf-brain/
```

### Les commandes ne sont pas loggées
```bash
# Vérifier que le hook est installé
cat ~/.ctf-brain/shell-hook.sh

# Vérifier le dossier de logs
ls -la ~/.ctf-brain/logs/
```

### Les variables ne sont pas chargées
```bash
# Vérifier le fichier .env
cat ~/.ctf-brain/boxes/box-1.env

# Tester manuellement
source ~/.ctf-brain/boxes/box-1.env
echo $CTF_IP
```

## 📝 Roadmap

**Phase 1 - MVP Interface** ✅
- [x] Vue liste des boxes
- [x] Vue détail avec notes et actions
- [x] Navigation clavier
- [x] Persistence JSON

**Phase 2 - Environment Management** ✅
- [x] Variables d'environnement custom
- [x] Lancement de shell automatique
- [x] Logging transparent des commandes
- [x] Génération de fichiers .env

**Phase 3 - Enrichissement** 📋
- [ ] Ajout/édition de notes depuis la TUI
- [ ] Catégorisation automatique des commandes
- [ ] Détection de succès/échec
- [ ] Timeline visuelle

**Phase 4 - Export** 📋
- [ ] Export writeup Markdown
- [ ] Export PDF
- [ ] Templates personnalisables

## 📄 License

MIT

## 👨‍💻 Auteur

**Salem GNK** - Étudiant en expertise informatique

