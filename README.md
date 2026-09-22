# skills

Daniel's personal agent skills. Straight from my `~/.agents` directory.

## Installation

**Claude Code (plugin):**

```bash
/plugin install danmondra-skills
```

**Any agent (editable copy):**

```bash
npx skills@latest add danmondra/skills
```

## Skills

| Skill | Description |
| ----- | ----------- |
| [`clase-grabada-interactiva`](./skills/personal/clase-grabada-interactiva/SKILL.md) | Interactive HTML study page built from a recorded class + transcript. |
| [`ensenar-paso-a-paso`](./skills/personal/ensenar-paso-a-paso/SKILL.md) | Enseña un tema bloque por bloque; el HTML solo recibe lo ya verificado como aprendido. Derivada de `teach` de Matt Pocock. |
| [`teach`](./skills/matt-pocock/teach/SKILL.md) | De Matt Pocock (verbatim): enseña un concepto nuevo dentro de un workspace con misión, recursos y learning records. |

## Agregar una skill (30 segundos)

**Skill propia nueva:**

```bash
mkdir -p skills/personal/mi-skill
# crea skills/personal/mi-skill/SKILL.md con frontmatter name: + description:
./scripts/link-skills.sh   # la linkea a ~/.agents/skills y ~/.claude/skills
```

Luego agrega su fila en este README y en `skills/personal/README.md`, y su ruta en `.claude-plugin/plugin.json` (`skills`). Valida con `claude plugin validate . --strict`, commitea y pushea.

**Copiar una skill ajena (verbatim):**

```bash
mkdir -p skills/<autor>/<skill>
cp -r ~/.agents/skills/<skill>/. skills/<autor>/<skill>/
```

Y lo mismo: fila en READMEs + entrada en `plugin.json` + validar + push.

## Descargar / instalar en otra máquina

**Como plugin (solo lectura, se actualiza solo):**

```bash
/plugin install danmondra-skills
```

**Como archivos editables:**

```bash
npx skills@latest add danmondra/skills   # el instalador deja elegir cuáles
# o clona el repo y linkea:
git clone https://github.com/danmondra/skills.git
cd skills && ./scripts/link-skills.sh
```
