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
cargo run --release
# ou
./target/release/ctf-brain
```

Au premier lancement, ctf-brain :

- Crée `~/.ctf-brain/`
- Installe le hook de logging (pour capturer les commandes)
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
#    Un shell s'ouvre avec toutes les variables et aliases utiles

# 6. Pour capturer une commande et son output dans le write-up :
[Lame] $ ctf nmap -sV $CTF_IP      # (ou cn)
[Lame] $ ctf gobuster ...          # (ou cg)
[Lame] $ echo $JWT_TOKEN           # Variable dispo

# 7. Tapez 'exit' pour revenir à l'app
#    Les commandes sont importées automatiquement dans la box

# 8. Générer le write-up Markdown (touche 'w' dans la vue Détails)
#    Le chemin du fichier généré s'affiche
```


### Contrôles clavier principaux

#### Vue Liste
| Touche         | Action                          |
| -------------- | ------------------------------- |
| `j` / `↓`      | Descendre dans la liste         |
| `k` / `↑`      | Monter dans la liste            |
| `Enter`        | Voir les détails               |
| `a`            | Ajouter une box                 |
| `d`            | Supprimer la box sélectionnée   |
| `l`            | Lancer shell avec environnement |
| `q`            | Quitter                         |

#### Vue Détails
| Touche  | Action                                |
| ------- | ------------------------------------- |
| `e`     | Éditer les variables d'environnement   |
| `n`     | Éditer les notes                      |
| `w`     | Générer le write-up Markdown          |
| `l`     | Lancer shell                          |
| `Esc`   | Retour à la liste                     |

#### Shell CTF (après 'l')
| Commande         | Action                                    |
|------------------|-------------------------------------------|
| `ctf <cmd>`      | Exécute et log la commande + output       |
| `cn`             | Alias: ctf nmap -sV $CTF_IP               |
| `cna`            | Alias: ctf nmap -sC -sV -A $CTF_IP        |
| `cg`             | Alias: ctf gobuster ...                   |
| `cff`            | Alias: ctf ffuf ...                       |

> **Astuce :** Utilisez toujours `ctf` pour les commandes importantes à documenter dans le write-up !

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

g /path    # gobuster dir -u http://$CTF_IP -w wordlist

## 🎯 Aliases et wrapper dans le shell

Quand tu lances un shell avec `l`, tu as accès à :

```bash
ctf <commande>   # Exécute et log la commande + output (pour le write-up)
cn               # Alias: ctf nmap -sV $CTF_IP
cna              # Alias: ctf nmap -sC -sV -A $CTF_IP
cg               # Alias: ctf gobuster ...
cff              # Alias: ctf ffuf ...
ip               # Affiche $CTF_IP
nc-listen        # rlwrap nc -lvnp 4444
```

> **Seules les commandes passées via `ctf` sont loggées avec leur output pour le write-up !**

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


## 📝 Fonctionnalités avancées

- **Import automatique des commandes** : Après chaque session shell, les commandes passées via `ctf` sont importées dans la box correspondante.
- **Génération de write-up** : Touche `w` dans la vue Détails → exporte un markdown structuré avec toutes les commandes, outputs et notes.
- **Aliases rapides** : Pour les outils classiques (nmap, gobuster, ffuf, etc).
- **Variables d'environnement** : Disponibles dans le shell pour chaque box.

## 📄 License

MIT

## 👨‍💻 Auteur

**Salem GNK** - Étudiant en expertise informatique
