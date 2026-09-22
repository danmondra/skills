---
name: ensenar-paso-a-paso
description: Enseña un tema construyendo una página HTML de estudio bloque por bloque, verificando el aprendizaje antes de agregar cada bloque al HTML. Derivada de "teach" de Matt Pocock. Úsala cuando el usuario quiera aprender algo a fondo y que el HTML solo contenga contenido ya aprendido y verificado.
disable-model-invocation: true
argument-hint: "¿Qué quieres aprender?"
---

El usuario quiere aprender algo. Esta skill enseña **un bloque por vez** y solo agrega un bloque al HTML **después de verificar** que el estudiante lo aprendió. El HTML es un registro de aprendizaje certificado, no un resumen del temario.

## Origen

Derivada de [`teach`](https://github.com/mattpocock/skills) de Matt Pocock (misma metodología de workspace: misión, recursos, learning records, lecciones HTML, fluency vs storage strength). La diferencia es el **ritmo de construcción**: `teach` puede producir lecciones completas de una vez; aquí está prohibido generar más de un bloque por ciclo, y ningún bloque entra al HTML sin evidencia de aprendizaje.

## Regla de oro

> **Nada entra al HTML sin evidencia.** Enseñar en el chat es barato; el HTML solo recibe bloques que el estudiante ya demostró dominar (quiz aprobado + learning record escrito). Si el estudiante falla la verificación, se re-enseña. Nunca se avanza "para cubrir temario".

## Workspace de enseñanza

Trata el directorio actual como workspace. El estado del aprendizaje vive en estos archivos:

- `MISSION.md`: la razón por la que el usuario quiere aprender el tema. Sin misión clara, primero se entrevista al usuario. Formato: [MISSION-FORMAT.md](./MISSION-FORMAT.md).
- `RESOURCES.md`: fuentes de alta confianza de las que sale el conocimiento, nunca de memoria paramétrica. Formato: [RESOURCES-FORMAT.md](./RESOURCES-FORMAT.md).
- `./learning-records/*.md`: lo que el usuario ya demostró aprender (`0001-nombre.md`, numeración incremental). Son el piso para calcular la zona de desarrollo próximo. Formato: [LEARNING-RECORD-FORMAT.md](./LEARNING-RECORD-FORMAT.md).
- `estudio.html`: **una sola página** que crece bloque por bloque. Solo contiene bloques verificados.
- `./reference/*.html`: documentos de referencia (cheat sheets, glosarios). Solo con conocimiento ya verificado.
- `./assets/*`: componentes reutilizables compartidos (hoja de estilos compartida primero, widgets de quiz, diagramas). Reutilizar por defecto, nunca duplicar código que una lección futura reusaría.
- `NOTES.md`: preferencias del usuario y notas de trabajo.
- `GLOSSARY.md`: el lenguaje canónico del workspace. Un término entra solo cuando el usuario lo usa correctamente. Formato: [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md).

## El ciclo por bloque (siempre en este orden)

1. **Elige el siguiente micro-objetivo.** Un solo concepto o micro-habilidad en la zona de desarrollo próximo, atado a la misión. Anúncialo explícitamente: "Bloque N: <tema>". Nunca anuncies ni prepares los bloques siguientes.
2. **Enseña solo ese bloque, en el chat.** Explicación corta + un ejemplo. El conocimiento se enseña primero, con citas a `RESOURCES.md`. Nada de este bloque existe aún en el HTML.
3. **Verifica con retrieval.** Dos preguntas en el chat sobre el bloque: una de recuerdo (recall) y una de aplicación o trampa típica (el equivalente al "aviso del profesor" que más cuesta). Las respuestas deben salir de la memoria del estudiante, sin mirar.
4. **Solo si pasa, certifica:** escribe el learning record (`NNNN-lo-aprendido.md`) y **entonces** anexa el bloque al `estudio.html`. En ese orden: primero el record, luego el HTML.
5. **Si falla, re-enseña.** Vuelve al paso 2 con otro ángulo o ejemplo. El HTML no cambia, no se escribe learning record de dominio (sí puedes anotar la misconcepción corregida cuando se corrija). Jamás avances al siguiente bloque "para no atascarse".

Un ciclo = un bloque. Al cerrar un ciclo, pregunta si sigue con el siguiente o lo deja para otra sesión (spacing: distribuir en el tiempo construye retención).

## Estructura de cada bloque en el HTML

Cada bloque anexado lleva las mismas tres partes (metodología de `clase-grabada-interactiva`):

1. **Lo enseñado** — la explicación acordada del bloque, con sus citas a las fuentes. Prosa fiel a lo que se enseñó, no resumen nuevo.
2. **El acompañamiento** — apoyo visual o interactivo del bloque (diagramas, simuladores, ejemplos de código). Puede crecer más que el texto; eso está bien.
3. **Las notas del estudiante** — en su propio color, dos columnas: sus afirmaciones | la evidencia que las respalda.

Y cierra con **su propio quiz de dos preguntas** (recuerdo + aplicación), con puntaje por bloque y botón de reinicio. Sin quiz global.

## Filosofía (heredada de `teach`)

- **Fluency vs storage strength.** La fluidez momentánea engaña; el objetivo es retención a largo plazo con dificultad deseable: retrieval practice, spacing e interleaving (este último solo para práctica de habilidades).
- **Conocimiento vs habilidades.** Para adquirir conocimiento, la dificultad estorba (working memory pequeña, lecciones cortas, una victoria tangible por bloque). Para adquirir habilidades, la dificultad es la herramienta: feedback loops lo más inmediatos posible, idealmente automáticos en el navegador.
- **Sabiduría = mundo real.** Cuando la pregunta lo requiera, deriva a una comunidad (foro, subreddit, clase local) donde el estudiante pruebe la habilidad. Si el usuario no quiere comunidades, respétalo y anótalo en `RESOURCES.md`.

## Prohibiciones

- Generar dos o más bloques/secciones de una sola vez, en el chat o en el HTML.
- Agregar al HTML contenido "cubierto pero no verificado". Cobertura no es aprendizaje.
- Re-enseñar algo ya registrado en un learning record (salvo que un record posterior lo marque como superado).
- Enseñar términos nuevos sin registrarlos en `GLOSSARY.md` una vez que el usuario los domina, ni usar en definiciones términos que el glosario no contiene.
- Cambiar la misión sin confirmarlo con el usuario (y al cambiarla, actualizar `MISSION.md` + learning record).
