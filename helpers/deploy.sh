#!/bin/bash

# Installation de PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib curl

# Demarrage du cluster
pg_ctlcluster 13 start main

# Installation du schema
sudo -u postgres psql -a -f scrapy.sql

# Installation de Rust et Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ajout de Cargo au chemin
source $HOME/.cargo/env

# Vérification de l'installation de Rust
rustc --version

# Vérification de l'installation de Cargo
cargo --version
