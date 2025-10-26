#!/bin/bash

# Script de instalação para mymed

set -e

echo "Instalando mymed..."

# Detectar arquitetura
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

if [ "$OS" = "linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        BINARY_URL="https://github.com/woulschneider/mymed/releases/download/v0.1.0/mymed-linux-x64"
    else
        echo "Arquitetura não suportada: $ARCH"
        exit 1
    fi
else
    echo "SO não suportado: $OS"
    exit 1
fi

# Baixar e instalar
curl -L "$BINARY_URL" -o /tmp/mymed
chmod +x /tmp/mymed
sudo mv /tmp/mymed /usr/local/bin/mymed

echo "Instalação concluída! Execute 'mymed --help' para ver os comandos."