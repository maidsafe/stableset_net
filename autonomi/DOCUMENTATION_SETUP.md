Below is a revised specification for setting up your MkDocs documentation structure and Jupyter integration, tailored to your existing directory layout:
 • /src/ (Rust code)
 • /python/ (Python code)
 • /nodejs/ (Node.js code)

We’ll keep these code folders intact and place our documentation in a separate /docs/ folder. This way, you can generate multi-language docs (including Jupyter notebooks) and have them reference code or examples from each of these subdirectories.

1. Updated Project Structure

Below is one way to organise your repo for the docs:

repo-root/
 ├─ src/               # Rust code
 │   └─ ...
 ├─ python/            # Python code
 │   └─ ...
 ├─ nodejs/            # Node.js code
 │   └─ ...
 ├─ docs/              # All documentation and notebooks
 │   ├─ index.md       # Main landing page
 │   ├─ rust/          # Rust-related docs or notebooks
 │   │   ├─ rust_tutorial.ipynb
 │   │   └─ code_samples.md
 │   ├─ python/        # Python docs & notebooks
 │   │   ├─ tutorial.ipynb
 │   │   └─ advanced_usage.md
 │   ├─ nodejs/        # Node.js docs & code examples
 │   │   ├─ index.md
 │   │   └─ code_samples.md
 │   └─ ...
 ├─ mkdocs.yml         # MkDocs config file
 └─ .github/
     └─ workflows/
         └─ build_docs.yml

Notes:
 • We keep /src/, /python/, and /nodejs/ purely for source code.
 • The /docs/ folder contains all the doc content (including notebooks).
 • Each language has its own subfolder under /docs/ for clarity.

2. MkDocs Installation & Basic Configuration

2.1 Installation

Make sure you have Python 3.7+. Install the required packages:

pip install mkdocs mkdocs-material mkdocs-jupyter

(mkdocs-material is optional but recommended for a nicer theme.)

2.2 mkdocs.yml Example

Create a file named mkdocs.yml in your repo root:

site_name: Safe Network Client Docs
site_description: Comprehensive multi-language client documentation

docs_dir: docs
site_dir: site

theme:
  name: material

plugins:

- search
- jupyter:
      execute: false  # or 'auto' if you'd like to run notebooks on each build

nav:

- Home: index.md
- Rust:
  - Rust Tutorial: rust/rust_tutorial.ipynb
  - Code Samples: rust/code_samples.md
- Python:
  - Tutorial: python/tutorial.ipynb
  - Advanced Usage: python/advanced_usage.md
- Node.js:
  - Overview: nodejs/index.md
  - Code Samples: nodejs/code_samples.md

Key Points:
 • nav defines the left-hand menu structure.
 • .ipynb files in the docs/ directory are automatically processed by mkdocs-jupyter.
 • If you want notebooks re-run at build time, set execute: auto.

3. Referencing Code in /src/, /python/, /nodejs/
1. Include Code Snippets
 • In your .md files or .ipynb notebooks, you can refer to code in your existing directories by copy-pasting the relevant lines or linking to them on GitHub.
 • For instance, you might do:

```rust
// Code snippet from /src/...




 2. Auto-Generating API References (optional)
 • Rust: cargo doc can generate documentation from /src/. If you want to integrate these HTML docs into your MkDocs site, you can place them in a subfolder like docs/rust-api/.
 • Python: Tools like Sphinx or pdoc can auto-generate doc pages from docstrings in /python/. You could store the generated output in docs/python-api/.
 • Node.js: TypeDoc or JSDoc can generate docs from JSDoc annotations in /nodejs/. Put the output in docs/nodejs-api/.

You can then link from your main mkdocs.yml nav to these generated folders (e.g., rust-api/index.html, etc.).

4. Python & Rust Notebooks

4.1 Python Notebooks
 • Put .ipynb files in docs/python/.
 • Code cells can import your package directly (e.g., if it’s installed in a virtual environment).
 • If you want to automatically run these notebooks each time you build, set execute: auto in mkdocs.yml. This ensures the examples always reflect the latest code behaviour.

4.2 Rust Notebooks (Optional)
 • If you truly want Rust notebooks, install the Evcxr kernel.
 • Otherwise, just store .md files with Rust code blocks:

```rust
// Example snippet referencing /src/ code
fn main() {
    println!("Hello from Rust!");
}




 • For interactive usage, consider linking to the Rust Playground or embedding an iframe if you want the user to run code live.

5. Node.js Examples
 1. Markdown Fenced Code Blocks
 • In docs/nodejs/code_samples.md:

```js
const safe = require('safe-network-client');
// Demonstrate usage
```

 2. Embedding RunKit
 • RunKit Embed Docs let you embed Node.js code as an interactive iframe.
 • Insert the HTML snippet in your Markdown:

<iframe
  width="100%"
  height="600"
  frameborder="0"
  src="https://runkit.com/e/embed?source=...">
</iframe>

 3. Codespaces / Codesandbox
 • Provide a link to a preconfigured environment for your Node.js library.
 • This is a popular option for larger code samples or complex setups.

5.1 Node.js Bindings Documentation

The Node.js bindings provide a TypeScript-based interface to the Autonomi client. The documentation should cover:

1. Installation & Setup

```bash
npm install @autonomi/client
```

2. TypeScript Configuration

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

3. Basic Usage

```typescript
import { Client, LinkedList, Pointer } from '@autonomi/client';

// Initialize client
const client = new Client();

// Create and store a linked list
const linkedList = new LinkedList();
const address = await client.linkedListPut(linkedList);

// Retrieve a linked list
const retrievedList = await client.linkedListGet(address);

// Work with pointers
const pointer = new Pointer();
const pointerAddress = await client.pointerPut(pointer);
```

4. API Reference

The Node.js bindings expose the following main classes and interfaces:

- `Client`: Main interface for interacting with the Autonomi network
  - `linkedListGet(address: LinkedListAddress): Promise<LinkedList>`
  - `linkedListPut(list: LinkedList): Promise<LinkedListAddress>`
  - `pointerGet(address: PointerAddress): Promise<Pointer>`
  - `pointerPut(pointer: Pointer): Promise<PointerAddress>`

- `LinkedList`: Represents a linked list data structure
  - Properties and methods for managing linked list data
  - Type-safe operations with TypeScript support

- `Pointer`: Represents a pointer in the network
  - Properties and methods for pointer management
  - Type-safe pointer operations

5. Examples

5.1 Creating and Managing Linked Lists

```typescript
import { Client, LinkedList } from '@autonomi/client';

async function example() {
  const client = new Client();
  
  // Create a new linked list
  const list = new LinkedList();
  
  // Add data to the list
  list.append("Hello");
  list.append("World");
  
  // Store the list
  const address = await client.linkedListPut(list);
  console.log(`List stored at: ${address}`);
  
  // Retrieve the list
  const retrieved = await client.linkedListGet(address);
  console.log(`Retrieved data: ${retrieved.toString()}`);
}
```

5.2 Working with Pointers

```typescript
import { Client, Pointer } from '@autonomi/client';

async function example() {
  const client = new Client();
  
  // Create a new pointer
  const pointer = new Pointer();
  
  // Set pointer data
  pointer.setTarget("example-target");
  
  // Store the pointer
  const address = await client.pointerPut(pointer);
  console.log(`Pointer stored at: ${address}`);
  
  // Retrieve the pointer
  const retrieved = await client.pointerGet(address);
  console.log(`Pointer target: ${retrieved.getTarget()}`);
}
```

6. Best Practices

- Always use TypeScript for better type safety and IDE support
- Handle errors appropriately using try/catch blocks
- Use async/await for all asynchronous operations
- Follow the provided examples for proper memory management
- Utilize the TypeScript compiler options as specified
- Keep the client instance for reuse rather than creating new instances

7. Testing

The Node.js bindings include a comprehensive test suite using Jest:

```typescript
import { Client } from '@autonomi/client';

describe('Client', () => {
  let client: Client;

  beforeEach(() => {
    client = new Client();
  });

  test('linked list operations', async () => {
    const list = new LinkedList();
    const address = await client.linkedListPut(list);
    const retrieved = await client.linkedListGet(address);
    expect(retrieved).toBeDefined();
  });
});
```

Run tests using:

```bash
npm test
```

6. GitHub Actions for Building & Deploying

Create .github/workflows/build_docs.yml:

name: Build and Deploy Docs

on:
  push:
    branches: [ "main" ]
  pull_request:

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v3

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.9'

      - name: Install Dependencies
        run: |
          pip install mkdocs mkdocs-material mkdocs-jupyter

      - name: Build Docs
        run: mkdocs build

      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          personal_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./site

Explanation:
 • On every commit to main (and PR), it checks out, installs MkDocs + plugins, builds your docs to site/, then deploys to GitHub Pages if on main.
 • In your GitHub repo settings, ensure GitHub Pages is enabled and configured for the gh-pages branch.

7. Contributing & Collaboration
1. CONTRIBUTING.md
 • Instruct community members to clone/fork the repo, then edit .md or .ipynb files within /docs/.
 • Show them how to install MkDocs dependencies, and run mkdocs serve locally for a live preview on <http://127.0.0.1:8000/>.
2. Style Guidelines
 • Decide on any style preferences (e.g., heading levels, code snippet formatting).
 • Possibly use a linter tool for Markdown or notebooks if you want consistent style.
3. Pull Request Workflows
 • Each PR triggers the build to ensure docs compile cleanly.
 • Merge once approved. The site auto-deploys on main.

8. Putting It All Together
1. Maintain Source Code in /src/, /python/, /nodejs/.
2. Create a /docs/ folder with subfolders for each language, plus a main index.md.
3. Set up mkdocs.yml with the jupyter plugin. Define your navigation.
4. Author Python & Rust notebooks (tutorial.ipynb etc.) and Node.js examples (code_samples.md).
5. Configure GitHub Actions to build and deploy your docs site automatically.
6. Encourage PR-based contributions for the community to enhance or fix the documentation.

By following this structure, you’ll have a clear separation of code and docs, an approachable set of Jupyter-based tutorials for Python (and possibly Rust), and straightforward Node.js examples—while still retaining a streamlined build and deployment pipeline for the docs.

Final Specification

 1. Use the folder structure in the snippet above.
 2. Install mkdocs, mkdocs-material, mkdocs-jupyter in your Python environment.
 3. Create and configure mkdocs.yml for your site name, theme, and nav.
 4. Author your docs:
 • Python notebooks under docs/python/
 • Rust docs/notebooks under docs/rust/
 • Node.js docs under docs/nodejs/ with code blocks or embedded interactive snippets.
 5. Add a GitHub Actions workflow (build_docs.yml) to automate building and optionally deploying on merges to main.
 6. Provide a CONTRIBUTING.md with instructions for local doc building (mkdocs serve) and the PR process.

With these steps implemented, you’ll have a robust, multi-language doc site that’s easy to maintain, expand, and keep in sync with your /src/, /python/, and /nodejs/ codebases.
