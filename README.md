# idea-board-rust

A Api that supports the post of new ideas and the community to handle it.

## API Routes

- /ideas
  - GET: list ideas
  - POST: create a new idea
- /ideas/:id
  - GET: find a idea by its ID
  - DELETE: delete a idea by its ID
- /ideas/:id/likes
  - GET: list all likes attached to a idea
  - POST: add +1 like to a idea
  - DELETE: add -1 like to a idea

## How to run

```bash

    make build
    make run

```
