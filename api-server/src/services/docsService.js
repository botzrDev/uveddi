const fs = require('fs-extra');
const path = require('path');
const marked = require('marked');
const matter = require('gray-matter');

class DocsService {
  constructor() {
    this.DOCS_PATH = path.join(__dirname, '..', '..', '..', 'docs');
  }

  async getDocumentationStructure() {
    const structure = await this.buildDirectoryStructure(this.DOCS_PATH);
    return { structure };
  }

  async getDocumentationFile(filePath) {
    const doc = await this.readMarkdownFile(filePath);
    return doc;
  }

  async searchDocumentation(query) {
    const structure = await this.buildDirectoryStructure(this.DOCS_PATH);
    const results = [];
    const searchQuery = query.toLowerCase();

    await this.searchInStructure(structure.structure, results, searchQuery);
    return results;
  }

  // Helper method to read and parse markdown files
  async readMarkdownFile(filePath) {
    try {
      const fullPath = path.join(this.DOCS_PATH, filePath);
      const fileContent = await fs.readFile(fullPath, 'utf8');

      const parsed = matter(fileContent);

      return {
        title: parsed.data.title || path.basename(filePath, '.md'),
        content: marked.parse(parsed.content),
        metadata: parsed.data,
        rawContent: parsed.content,
        path: filePath
      };
    } catch (error) {
      console.error('Error reading markdown file:', error);
      return null;
    }
  }

  // Helper method to build directory structure
  async buildDirectoryStructure(dirPath, basePath = '') {
    try {
      const items = await fs.readdir(dirPath);
      const structure = [];

      for (const item of items) {
        const fullPath = path.join(dirPath, item);
        const stats = await fs.stat(fullPath);
        const relativePath = path.join(basePath, item);

        if (stats.isDirectory()) {
          const children = await this.buildDirectoryStructure(fullPath, relativePath);
          structure.push({
            name: item,
            path: relativePath,
            type: 'directory',
            children: children.structure
          });
        } else if (item.endsWith('.md')) {
          const doc = await this.readMarkdownFile(relativePath);
          structure.push({
            name: item,
            path: relativePath,
            type: 'file',
            title: doc?.title || item,
            size: stats.size
          });
        }
      }

      return { structure };
    } catch (error) {
      console.error('Error building directory structure:', error);
      return { structure: [] };
    }
  }

  // Helper method to search within structure
  async searchInStructure(items, results, query) {
    for (const item of items) {
      if (item.type === 'file' && item.name.endsWith('.md')) {
        const doc = await this.readMarkdownFile(item.path);
        if (doc && (
          doc.rawContent.toLowerCase().includes(query) ||
          item.title.toLowerCase().includes(query)
        )) {
          results.push({
            title: item.title,
            path: item.path,
            snippet: doc.rawContent.substring(0, 200) + '...'
          });
        }
      } else if (item.type === 'directory' && item.children) {
        await this.searchInStructure(item.children, results, query);
      }
    }
  }
}

module.exports = DocsService;