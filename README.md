# IRCChannel

[![Tests](https://github.com/ditr4drian/IRCChannel/actions/workflows/tests.yml/badge.svg)](https://github.com/ditr4drian/IRCChannel/actions/workflows/tests.yml)
[![Release](https://img.shields.io/github/v/release/ditr4drian/IRCChannel)](https://github.com/ditr4drian/IRCChannel/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Un chat de equipo sencillo, rápido y privado que vive en tu terminal.**

IRCChannel te permite platicar con tu equipo desde la misma ventana donde ya trabajas, sin abrir el navegador ni instalar aplicaciones pesadas. Se conecta a cualquier servidor de chat **IRC**, un sistema de mensajería abierto que existe desde hace décadas y que tu equipo puede alojar en su propia red.

```
┌ #equipo ───────────────────────────────────────────────┐
│ 09:12 ana   buenas, ya estoy conectada                 │
│ 09:13 beto  ¿alguien revisó el servidor de pruebas?    │
│ 09:13 tú    yo lo veo en un rato 👍                     │
└────────────────────────────────────────────────────────┘
 > escribe tu mensaje aquí…
```

## ¿Por qué usarlo?

- **Privado:** las conversaciones se quedan en el servidor de tu equipo, no en la nube de alguien más.
- **Ligero:** consume muy poca memoria y abre al instante.
- **Sin distracciones:** solo texto. Sin anuncios, sin notificaciones infinitas.
- **Varios canales:** entra a varias salas a la vez y cambia entre ellas con una tecla.
- **Tu hora local:** los mensajes muestran la hora en tu zona horaria.
- **Libre y gratuito:** código abierto con licencia MIT.

## Instalación

### Opción 1: descarga directa (Linux)

Descarga el programa ya listo desde la [última versión publicada](https://github.com/ditr4drian/IRCChannel/releases/latest):

```bash
curl -L -o ircchannel \
  https://github.com/ditr4drian/IRCChannel/releases/latest/download/ircchannel-linux-x86_64
chmod +x ircchannel
sudo mv ircchannel /usr/local/bin/
```

### Opción 2: instalar desde el código

Necesitas tener instalado **Rust** (el lenguaje con el que está hecho). Si no lo tienes, sigue las instrucciones de [rustup.rs](https://rustup.rs); solo toma un par de minutos.

Después, en tu terminal:

```bash
cargo install --git https://github.com/ditr4drian/IRCChannel
```

Listo. Ya puedes abrirlo escribiendo:

```bash
ircchannel
```

## Configuración

Antes de la primera conexión, dile a IRCChannel a qué servidor conectarse y cómo te llamas.

1. Crea la carpeta de configuración y copia el ejemplo:

   ```bash
   mkdir -p ~/.config/ircchannel
   curl -o ~/.config/ircchannel/config.toml \
     https://raw.githubusercontent.com/ditr4drian/IRCChannel/main/config/config.example.toml
   ```

2. Abre `~/.config/ircchannel/config.toml` con tu editor favorito y cambia:
   - `host`: la dirección del servidor de tu equipo.
   - `nick`: el nombre con el que te verán los demás.
   - `channels`: las salas a las que quieres entrar.
   - `timezone`: tu zona horaria (por ejemplo `America/Mexico_City` o `Europe/Madrid`).

Así se ve el archivo:

```toml
[connection]
host = "irc.example.com"
port = 6667
server_password = ""

[identity]
nick = "tu_nombre"
username = "tu_nombre"
realname = "Tu Nombre"

[session]
channels = ["#general", "#equipo"]
send_delay_ms = 0

[ui]
tick_ms = 250
history_limit = 2000
timezone = "America/Mexico_City"
```

> Deja todas las secciones en el archivo, aunque no cambies nada en ellas.

## Cómo se usa

Escribe tu mensaje y presiona **Enter** para enviarlo al canal activo.

| Tecla | Qué hace |
|---|---|
| `Enter` | Enviar mensaje |
| `Tab` / `Shift+Tab` | Cambiar de canal |
| `RePág` / `AvPág` | Ver mensajes anteriores |
| `Ctrl+C` o `Ctrl+Q` | Salir |

También puedes escribir comandos que empiezan con `/`:

| Comando | Qué hace |
|---|---|
| `/join #sala` | Entrar a una sala (si no existe, se crea) |
| `/part` | Salir de la sala actual |
| `/msg nombre hola` | Mensaje privado a una persona |
| `/nick nuevo_nombre` | Cambiar tu nombre |
| `/quit` | Desconectarte y salir |
| `/help` | Ver todos los comandos |

## Preguntas frecuentes

**¿Necesito un servidor?**
Sí. IRCChannel es la aplicación con la que chateas; el servidor es el lugar donde se reúnen todos. Tu equipo puede usar uno público o montar uno propio (por ejemplo, [Ergo](https://ergo.chat) es fácil de instalar).

**¿Las salas son permanentes como en Discord o Slack?**
En IRC, normalmente una sala existe mientras haya alguien dentro, y los mensajes no se guardan para quien llega después. Algunos servidores permiten registrar salas para que sean permanentes.

**¿Funciona en Windows, macOS y Linux?**
Está pensado y probado en Linux. En macOS y Windows debería funcionar, pero la carpeta de configuración cambia: en macOS es `~/Library/Application Support/mx.ditra.ircchannel/` y en Windows `%APPDATA%\ditra\ircchannel\config\`.

**Algo no funciona, ¿dónde veo qué pasó?**
IRCChannel guarda un registro en `~/.local/share/ircchannel/ircchannel.log`. Si quieres reportar un problema, abre un [issue](https://github.com/ditr4drian/IRCChannel/issues) y adjunta lo que veas ahí.

## Novedades

Cada versión nueva y sus cambios quedan registrados en el [CHANGELOG](CHANGELOG.md) y en la página de [Releases](https://github.com/ditr4drian/IRCChannel/releases).

## Para desarrolladores

Si quieres entender cómo está construido o contribuir, revisa [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Licencia

Distribuido bajo la licencia [MIT](LICENSE). Puedes usarlo, modificarlo y compartirlo libremente.
