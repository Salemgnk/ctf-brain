# 🧩 CTF Brain - Cahier des Charges

## 📌 Informations Générales

**Nom du projet** : CTF Brain  
**Version** : 0.2.0 (MVP + Environment Management)  
**Langage** : Rust  
**Type** : Application TUI (Terminal User Interface)  
**Auteur** : Salemgnk  
**Licence** : MIT  

---

## 🎯 Contexte et Problématique

### Problème identifié

Les participants de CTF (Capture The Flag) et de plateformes comme HackTheBox, picoCTF, TryHackMe font face à plusieurs défis :

1. **Perte de contexte** : Lors de sessions longues, on oublie quelles commandes ont été testées
2. **Répétition manuelle** : Retaper constamment l'IP de la machine cible
3. **Notes dispersées** : Les découvertes sont notées dans des fichiers texte éparpillés
4. **Difficulté à retracer** : Impossible de reconstruire facilement le cheminement pour un writeup
5. **Gestion multi-fenêtres** : Travailler avec plusieurs terminaux sans contexte partagé

### Solution proposée

CTF Brain est un **gestionnaire de sessions CTF intelligent** qui combine :
- Une interface TUI moderne pour gérer les machines
- Un système d'environnement shell automatique avec logging transparent
- Une base de données locale pour persister les informations
- Un mécanisme de variables d'environnement partagées entre terminaux

---

## ✨ Fonctionnalités Actuelles

### Phase 1 : Interface de Gestion (✅ Implémenté)

#### 1.1 Gestion des Boxes CTF

**Description** : Créer, lister, consulter et supprimer des machines CTF

**Fonctionnalités** :
- ✅ **Liste des boxes** avec icônes par plateforme (HTB 🔴, picoCTF 🎯, TryHackMe 🟢)
- ✅ **Navigation au clavier** (j/k ou flèches)
- ✅ **Ajout de box** via formulaire modal
  - Champs : Titre, Plateforme, IP, Tags
  - Validation d'IP
  - Navigation Tab/Shift+Tab entre champs
- ✅ **Suppression avec confirmation**
- ✅ **Persistence JSON** automatique dans `~/.local/share/ctf-brain/boxes.json`

**Modèle de données** :
```rust
struct CtfBox {
    id: i32,
    title: String,
    platform: String,
    ip_address: IpAddr,
    tags: Vec<String>,
    created_date: DateTime<Utc>,
    updated_date: DateTime<Utc>,
    actions: Vec<Action>,
    notes: Vec<Note>,
    env_vars: HashMap<String, String>,  // Phase 2
}
```

#### 1.2 Vue Détails

**Description** : Consultation approfondie d'une box

**Affichage** :
- Informations générales (IP, plateforme, tags)
- Liste des notes catégorisées (Web 🌐, Pwn 💥, Crypto 🔐, Recon 🔍, etc.)
- Historique des actions/commandes
- Dates de création et modification

**Actions disponibles** :
- `Esc` : Retour à la liste
- `e` : Éditer les variables d'environnement (Phase 2)
- `l` : Lancer un shell avec environnement chargé (Phase 2)

#### 1.3 Système de Notes

**Description** : Organisation des découvertes par catégorie

**Catégories disponibles** :
- `Recon` : Reconnaissance initiale
- `Foothold` : Point d'entrée
- `Privesc` : Élévation de privilèges
- `Web` : Vulnérabilités web
- `Pwn` : Exploitation binaire
- `Crypto` : Cryptographie
- `Reversing` : Reverse engineering
- `Stego` : Stéganographie
- `Misc` : Divers

**État actuel** : Affichage uniquement (données de sample)

#### 1.4 Tracking d'Actions

**Description** : Historique des commandes exécutées

**Modèle** :
```rust
struct Action {
    timestamp: DateTime<Utc>,
    command: String,
    result: ActionResult,  // Success, Fail, Unknown
    note: Option<String>,
}
```

**État actuel** : Affichage de données de sample

---

### Phase 2 : Environment Management (🚧 En cours d'implémentation)

#### 2.1 Génération d'Environnement Shell

**Description** : Création automatique de fichiers d'environnement par box

**Localisation** : `~/.ctf-brain/boxes/box-{id}.env`

**Contenu généré** :
```bash
#!/bin/bash
# Variables CTF
export CTF_BOX="Lame"
export CTF_IP="10.10.10.3"
export CTF_ID="1"
export CTF_PLATFORM="HTB"

# Variables custom utilisateur
export JWT_TOKEN="eyJhbGci..."
export API_KEY="sk-proj-..."

# Hook de logging
source ~/.ctf-brain/shell-hook.sh

# Prompt personnalisé
PS1="\[\e[32m\][Lame]\[\e[0m\] \u@\h:\w\$ "

# Aliases rapides
alias ip='echo $CTF_IP'
alias n='nmap -sV $CTF_IP'
alias na='nmap -sC -sV -A $CTF_IP'
alias g='gobuster dir -u http://$CTF_IP'
alias nc-listen='rlwrap nc -lvnp 4444'
```

**Déclencheur** : Lors de la sélection d'une box (touche `l`)

#### 2.2 Lancement de Shell Automatique

**Description** : Remplacer le process ctf-brain par un shell bash avec environnement chargé

**Mécanisme technique** :
```rust
// Utilisation de exec() UNIX
Command::new("bash")
    .arg("--rcfile")
    .arg("~/.ctf-brain/boxes/box-1.env")
    .exec();  // Remplace le process, ne retourne jamais
```

**Workflow utilisateur** :
1. Lancer `ctf-brain` dans un terminal
2. Sélectionner une box (touches j/k)
3. Appuyer sur `l` (Launch)
4. → ctf-brain se ferme, remplacé par bash avec env chargé
5. Prompt devient : `[Lame] user@host:~$`

**Avantage** : Pas de daemon, pas de process background, juste un remplacement propre

#### 2.3 Logging Automatique des Commandes

**Description** : Capture transparente de toutes les commandes shell

**Mécanisme** : Hook bash via `PROMPT_COMMAND` ou `trap DEBUG`

**Fichier** : `~/.ctf-brain/shell-hook.sh`

**Principe** :
```bash
_ctf_log_command() {
    local timestamp=$(date -Iseconds)
    local log_file="$HOME/.ctf-brain/logs/box-${CTF_ID}.jsonl"
    
    echo "{\"time\":\"$timestamp\",\"box_id\":$CTF_ID,\"cmd\":\"$1\"}" >> "$log_file"
}

# Hook bash
trap '_previous_command=$_this_command; _this_command=$BASH_COMMAND' DEBUG
PROMPT_COMMAND='_ctf_log_command "$_previous_command"'
```

**Format de log** : JSONL (JSON Lines) pour parsing facile
```json
{"time":"2026-02-08T14:23:45+00:00","box_id":1,"cmd":"nmap -sV 10.10.10.3"}
{"time":"2026-02-08T14:24:12+00:00","box_id":1,"cmd":"gobuster dir -u http://10.10.10.3"}
```

**Filtrage** : Ignore les commandes internes (cd, ls, pwd, clear)

#### 2.4 Variables d'Environnement Custom

**Description** : Stockage de tokens, cookies, clés API par box

**Use case** :
```bash
# Tu exploites et récupères un JWT
[Lame] $ curl http://$CTF_IP/api/login
{"token": "eyJhbGci..."}

# Tu retournes dans ctf-brain (Ctrl+Z ou nouveau terminal)
$ ctf-brain
# Vue détails → Touche 'e' (edit env)

# Tu ajoutes :
# KEY: JWT_TOKEN
# VALUE: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# Relance le shell (touche 'l')
[Lame] $ echo $JWT_TOKEN
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

[Lame] $ curl -H "Authorization: Bearer $JWT_TOKEN" http://$CTF_IP/api/admin
```

**Interface** :
- Vue `EditEnvVars` avec liste des variables actuelles
- Formulaire d'ajout : KEY + VALUE
- Validation : clé alphanumeric + underscore uniquement
- Suppression possible
- Sauvegarde automatique dans `boxes.json`

**Génération** : Variables exportées dans les fichiers `.env`

#### 2.5 Multi-Terminal avec Contexte Partagé

**Description** : Plusieurs terminaux peuvent travailler sur la même box

**Workflow** :
```
Terminal 1 (Konsole pane 1):
$ ctf-brain
# Sélectionne "Lame" → 'l'
[Lame] $ nmap -sV $CTF_IP

Terminal 2 (Konsole pane 2 - split manuel):
$ ctf-brain
# Sélectionne "Lame" → 'l'
[Lame] $ gobuster dir -u http://$CTF_IP

Terminal 3 (Konsole pane 3):
$ ctf-brain
# Vue détails de "Lame"
# Voir en temps réel les commandes des autres panes
```

**Mécanisme** :
- Chaque terminal charge le même fichier `box-1.env`
- Les commandes sont loggées dans le même `box-1.jsonl`
- Le fichier JSON est relu périodiquement dans la TUI (polling)

---

## 🏗️ Architecture Technique

### Structure du Projet

```
ctf-brain/
├── src/
│   ├── main.rs                 # Point d'entrée, event loop
│   ├── app.rs                  # Logique applicative, état
│   ├── models/
│   │   ├── mod.rs
│   │   ├── box.rs             # Structure CtfBox
│   │   ├── action.rs          # Structure Action
│   │   └── note.rs            # Structure Note
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── storage.rs         # Load/Save JSON
│   │   └── environment.rs     # Génération .env (Phase 2)
│   └── ui/
│       ├── mod.rs
│       ├── list.rs            # Vue liste
│       ├── detail.rs          # Vue détails
│       ├── add_box.rs         # Modal ajout
│       ├── delete_box.rs      # Modal suppression
│       └── edit_env_vars.rs   # Modal variables (Phase 2)
├── assets/
│   └── shell-hook.sh          # Script bash de logging
├── Cargo.toml
└── README.md
```

### Architecture des Données

```
~/.ctf-brain/
├── boxes.json                  # Base de données (serde_json)
├── boxes/
│   ├── box-1.env              # Env shell pour "Lame"
│   ├── box-2.env              # Env shell pour "WebGauntlet"
│   └── box-3.env
├── logs/
│   ├── box-1.jsonl            # Logs de commandes "Lame"
│   └── box-2.jsonl
└── shell-hook.sh              # Hook bash (copié au premier lancement)
```

### Pattern MVC-like

```
┌─────────────┐
│   main.rs   │  Event loop (crossterm)
│  (Controller)│
└──────┬──────┘
       │
       ├──────> ┌──────────┐
       │        │  app.rs  │  État + logique métier
       │        │  (Model) │
       │        └──────────┘
       │
       └──────> ┌──────────┐
                │   ui/    │  Rendu ratatui
                │  (View)  │
                └──────────┘
```

### Flow d'Exécution

#### Lancement normal
```
1. main.rs
   ├─> storage::load_boxes()
   ├─> App::new(boxes)
   ├─> enable_raw_mode()
   ├─> Terminal::new()
   └─> Event loop
       ├─> terminal.draw()
       ├─> event::poll()
       ├─> handle input
       └─> loop...
```

#### Lancement de shell
```
1. User appuie sur 'l'
2. app.launch_box_shell(box_id)
   ├─> storage::create_box_environment(box)
   │   └─> Écrit ~/.ctf-brain/boxes/box-1.env
   ├─> disable_raw_mode()
   └─> Command::new("bash").arg("--rcfile").exec()
       └─> Remplace le process ctf-brain
```

#### Logging de commandes
```
1. User tape "nmap -sV 10.10.10.3" dans bash
2. PROMPT_COMMAND trigger
3. shell-hook.sh::_ctf_log_command()
   └─> Append à ~/.ctf-brain/logs/box-1.jsonl
```

---

## 🔧 Stack Technique

### Dépendances Cargo

```toml
[dependencies]
# TUI
ratatui = "0.26"
crossterm = "0.27"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Dates
chrono = { version = "0.4", features = ["serde"] }

# Filesystem
directories = "5.0"

# Error handling
anyhow = "1.0"
```

### Concepts Rust Utilisés

#### 1. Ownership & Borrowing
```rust
// Référence mutable
pub fn next(&mut self)

// Référence immuable
pub fn save_boxes(boxes: &[CtfBox])

// Ownership transféré
pub fn new(boxes: Vec<CtfBox>) -> Self
```

#### 2. Pattern Matching
```rust
match key.code {
    KeyCode::Char('q') => app.quit(),
    KeyCode::Enter => app.select_current(),
    _ => {}
}
```

#### 3. Result & Option
```rust
pub fn load_boxes() -> Result<Vec<CtfBox>>
pub fn selected_box_id: Option<i32>

// Propagation d'erreur
let boxes = storage::load_boxes()?;
```

#### 4. Enums avec données
```rust
enum AppView {
    List,
    Details(i32),        // Porte un ID
    DeleteBox(i32),
    EditEnvVars(i32),
}
```

#### 5. Traits & Derive macros
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CtfBox { ... }
```

#### 6. Collections
```rust
Vec<CtfBox>                    // Tableau dynamique
HashMap<String, String>        // Map clé-valeur
```

---

## 📦 Installation et Utilisation

### Prérequis

- Rust 1.70+ (`rustup`)
- Terminal compatible UTF-8
- Bash ou Zsh

### Installation

```bash
# Cloner le repo
git clone https://github.com/Salemgnk/ctf-brain.git
cd ctf-brain

# Compiler en mode release
cargo build --release

# Installer (optionnel)
cargo install --path .
```

### Premier Lancement

```bash
# Lancer l'application
cargo run

# Ou si installé
ctf-brain
```

**Au premier lancement** :
1. Création de `~/.ctf-brain/`
2. Installation de `shell-hook.sh`
3. Affichage de données de sample (3 boxes)

### Contrôles Clavier

#### Vue Liste
| Touche | Action |
|--------|--------|
| `j` / `↓` | Descendre |
| `k` / `↑` | Monter |
| `Enter` | Voir détails |
| `a` | Ajouter une box |
| `d` | Supprimer la box sélectionnée |
| `l` | Lancer shell avec env (Phase 2) |
| `q` | Quitter |

#### Vue Détails
| Touche | Action |
|--------|--------|
| `Esc` | Retour à la liste |
| `e` | Éditer variables d'env (Phase 2) |
| `l` | Lancer shell (Phase 2) |

#### Formulaire Ajout
| Touche | Action |
|--------|--------|
| `Tab` | Champ suivant |
| `Shift+Tab` | Champ précédent |
| `Enter` | Valider |
| `Esc` | Annuler |
| `Backspace` | Effacer |

#### Modal Suppression
| Touche | Action |
|--------|--------|
| `y` | Confirmer |
| `n` / `Esc` | Annuler |

### Workflow Typique

```bash
# 1. Lancer ctf-brain
$ ctf-brain

# 2. Ajouter une box (touche 'a')
#    Titre: Lame
#    Platform: HTB
#    IP: 10.10.10.3
#    Tags: easy, linux

# 3. Sélectionner la box (Enter pour détails)

# 4. Ajouter des variables si nécessaire (touche 'e')
#    JWT_TOKEN = eyJhbGci...
#    API_KEY = sk-proj-...

# 5. Lancer le shell (touche 'l')
[Lame] $ n                    # Alias pour nmap -sV $CTF_IP
[Lame] $ echo $JWT_TOKEN      # Variable custom disponible

# 6. Split Konsole manuellement

# 7. Dans nouveau pane:
$ ctf-brain
# Sélectionne "Lame" → 'l'
[Lame] $ g /admin             # Alias pour gobuster

# 8. Retour dans ctf-brain pour voir logs
$ ctf-brain
# Vue détails → Historique mis à jour en temps réel
```

---

## 🚀 Roadmap

### Phase 2 - Environment Management (🚧 En cours)

- [x] Modèle de données avec `env_vars: HashMap`
- [ ] Génération de fichiers `.env`
- [ ] Fonction `launch_box_shell()`
- [ ] Script `shell-hook.sh` de logging
- [ ] UI `edit_env_vars.rs`
- [ ] Import des logs JSONL dans la TUI
- [ ] Rafraîchissement live des actions

### Phase 3 - Enrichissement (📋 Planifié)

- [ ] Ajout/édition de notes depuis la TUI
- [ ] Catégorisation automatique des commandes
  - nmap → Recon
  - gobuster → Web
  - msfconsole → Pwn
- [ ] Détection de succès/échec des commandes
  - Parser les codes de retour ($?)
  - Regex sur output (Found, 404, Access Denied)
- [ ] Timeline visuelle des actions
- [ ] Recherche/filtrage de boxes par tags

### Phase 4 - Export & Reporting (📋 Planifié)

- [ ] Export writeup en Markdown
  ```markdown
  # Lame - HackTheBox
  
  ## Reconnaissance
  ```bash
  nmap -sV 10.10.10.3
  ```
  Found open ports: 21, 22, 445
  
  ## Foothold
  ...
  ```
- [ ] Export PDF
- [ ] Templates de writeup personnalisables
- [ ] Génération automatique de timeline

### Phase 5 - Intelligence (🔮 Futur)

- [ ] Suggestions de pistes basées sur l'historique
  - "Tu as trouvé un port SMB, essaye CVE-2017-7494"
- [ ] Détection de patterns
  - "3 tentatives échouées → essaye autre chose"
- [ ] Base de connaissances locale
  - Liens vers PayloadsAllTheThings
  - Cheatsheets intégrés
- [ ] Mode collaboration
  - Partage de sessions entre équipe

---

## 🎓 Aspects Pédagogiques

### Concepts Rust Appris

1. **Gestion mémoire** : Ownership, borrowing, lifetimes
2. **Pattern matching** : Puissance des `match` et `if let`
3. **Error handling** : `Result<T, E>` et `?` operator
4. **Collections** : `Vec`, `HashMap`, iterators
5. **Traits** : `Serialize`, `Deserialize`, `Clone`
6. **Process spawning** : `Command::new().exec()`
7. **File I/O** : `fs::read_to_string`, `fs::write`

### Bonnes Pratiques

- ✅ Séparation des responsabilités (MVC)
- ✅ Tests unitaires (à venir)
- ✅ Documentation inline
- ✅ Error handling exhaustif
- ✅ Pas de `unwrap()` en production
- ✅ Logging des erreurs avec `eprintln!`

---

## 🐛 Problèmes Connus

### Limitations Actuelles

1. **Données de sample en dur** : Les notes et actions sont statiques
2. **Pas de gestion d'erreur UI** : Les erreurs s'affichent dans stderr
3. **Pas de tests** : Couverture de tests à 0%
4. **UI non responsive** : Peut casser sur terminaux < 80x24
5. **Mono-utilisateur** : Pas de gestion de permissions
6. **Bash uniquement** : Zsh fonctionne mais non testé

### Bugs à Corriger

- [ ] Cursor clignote après exec() de bash
- [ ] Rafraîchissement des logs pas encore implémenté
- [ ] Validation IP accepte les valeurs hors range
- [ ] Pas de confirmation avant quit si modifications non sauvées

---

## 🤝 Contribution

### Comment Contribuer

1. Fork le projet
2. Créer une branche feature (`git checkout -b feature/AmazingFeature`)
3. Commit (`git commit -m 'Add AmazingFeature'`)
4. Push (`git push origin feature/AmazingFeature`)
5. Ouvrir une Pull Request

### Guidelines

- Respecter le style Rust (rustfmt)
- Ajouter des tests pour les nouvelles features
- Mettre à jour le README si nécessaire
- Documenter les fonctions publiques

---

## 📄 Licence

MIT License - Voir fichier LICENSE

---

## 👨‍💻 Auteur

**Salem GNK**
- Étudiant en expertise informatique (3e année)
- Passionné de cybersécurité
- Rêve : Trouver une bounty 🎯

---

## 🙏 Remerciements

- HackTheBox pour l'inspiration
- La communauté Rust pour les libs incroyables
- Tous les CTF players qui testent l'outil

---

*Dernière mise à jour : 8 février 2026*
