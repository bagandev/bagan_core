// Liquid
export type LiquidPlugin = (engine:liquid.LiquidEngine) => void;
/**
 * Which binding artifact the generated loader actually loaded: `'native'` for
 * a native addon, otherwise the `platformArchABI` of the WASI flavor. Every
 * flavor napi-rs can build is listed, because `NAPI_RS_NATIVE_LIBRARY_PATH`
 * can point the loader at a WASI artifact this package does not build itself.
 */
export declare const __napiBindingTarget: 'native' | 'wasm32-wasi' | 'wasm32-wasip1'

export declare namespace ignore {
  export class IgnoreMatcher {
    constructor(patterns: Array<string>)
    /** Exported to JS as `isIgnored(path: string): boolean` */
    isIgnored(path: string): boolean
    /** Optional batch helper: exported to JS as `filterIgnored(paths: string[]): string[]` */
    filterIgnored(paths: Array<string>): Array<string>
  }
}

export declare namespace liquid {
  /**
   * A Liquid renderer that can be extended with JavaScript callbacks.
   *
   * Create an engine, optionally register custom behavior or partial sources,
   * then call [`render`](LiquidEngine::render) with a template and JSON context.
   */
  export class LiquidEngine {
    /**
     * Creates an engine with the standard Liquid filters and tags.
     *
     * # Arguments
     *
     * * `include_priority` - Determines whether the partial loader or the
     *   filesystem is checked first when both are configured. Defaults to
     *   `LoaderFirst`.
     */
    constructor(includePriority?: "LoaderFirst" | "FileSystemFirst")
    /**
     * Registers a JavaScript callback as a Liquid filter.
     *
     * The callback receives the input value followed by an array of
     * positional arguments, and may return a value or a promise resolving to
     * a value. Registering the filter invalidates the parser and template
     * cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `name` - Name used to invoke the filter in a Liquid template.
     * * `callback` - Function that transforms the input value.
     *
     * # Errors
     *
     * Returns an error if the callback cannot be converted to a
     * thread-safe function.
     */
    registerFilter(name: string, callback: (input:any,args:any[]) => Promise<any>): liquid.LiquidEngine
    /**
     * Registers a JavaScript callback as a Liquid tag.
     *
     * The callback receives an array of evaluated tag arguments and may
     * return a string or a promise resolving to one. Registering the tag
     * invalidates the parser and template cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `name` - Tag name used in Liquid templates.
     * * `callback` - Function that produces the tag's rendered output.
     *
     * # Errors
     *
     * Returns an error if the callback cannot be converted to a
     * thread-safe function.
     */
    registerTag(name: string, callback: (args:any[]) => Promise<string> | string): liquid.LiquidEngine
    /**
     * Registers a JavaScript callback as a Liquid block.
     *
     * The callback receives the rendered block body and an array of evaluated
     * arguments, and may return a string or a promise resolving to one. The
     * closing tag is named `end{name}`. Registering the block invalidates the
     * parser and template cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `name` - Opening block tag name.
     * * `callback` - Function that transforms the rendered body.
     *
     * # Errors
     *
     * Returns an error if the callback cannot be converted to a
     * thread-safe function.
     */
    registerBlock(name: string, callback: (body:string,args:any[]) => Promise<string> | string): liquid.LiquidEngine
    /**
     * Sets the directory used to load partial templates from the filesystem.
     *
     * Partial names are resolved relative to this directory. If a custom
     * partial loader is also registered, the configured include priority
     * determines which source is tried first. This invalidates the parser and
     * template cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `dir` - Filesystem directory containing partial templates.
     */
    setPartialsDir(dir: string): liquid.LiquidEngine
    /**
     * Sets the directory used to load layout templates from the filesystem.
     *
     * Layout names are resolved relative to this directory. If a custom
     * layout loader is also registered, the layout priority determines which
     * source is tried first.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `dir` - Filesystem directory containing layout templates.
     */
    setLayoutsDir(dir: string): liquid.LiquidEngine
    /**
     * Sets the extensions tried when resolving layouts without an extension.
     *
     * Leading dots are removed from each extension. The default order is
     * `liquid`, then `html`. An empty list leaves no fallback extension.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `extensions` - Extensions to try, in priority order.
     */
    setLayoutExtensions(extensions: Array<string>): liquid.LiquidEngine
    /**
     * Sets the extensions tried when resolving partials without an extension.
     *
     * Leading dots are removed from each extension. The default order is
     * `liquid`, then `html`. Changing the extensions invalidates the parser
     * and template cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `extensions` - Extensions to try, in priority order.
     */
    setPartialExtensions(extensions: Array<string>): liquid.LiquidEngine
    /**
     * Registers a JavaScript callback to load partial templates.
     *
     * The callback receives a partial name and may resolve to the partial's
     * source text or `None` if it is not available. If a filesystem directory
     * is also configured, the include priority determines which source is
     * tried first. Registering the loader invalidates the parser and template
     * cache.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `callback` - Async function that looks up partial source by name.
     *
     * # Errors
     *
     * Returns an error if the callback cannot be converted to a
     * thread-safe function.
     */
    registerPartialLoader(callback: (name:string) => Promise<string> | null | undefined): liquid.LiquidEngine
    /**
     * Registers a JavaScript callback to load layout templates.
     *
     * The callback receives a layout name and may resolve to the layout's
     * source text or `None` if it is not available. If a filesystem directory
     * is also configured, the layout priority determines which source is
     * tried first.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `callback` - Async function that looks up layout source by name.
     *
     * # Errors
     *
     * Returns an error if the callback cannot be converted to a
     * thread-safe function.
     */
    registerLayoutLoader(callback: (name:string) => Promise<string> | null | undefined): liquid.LiquidEngine
    /**
     * Sets whether the loader or filesystem is tried first for layouts.
     *
     * The default is `LoaderFirst`.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `priority` - Source to check first when both are configured.
     */
    setLayoutPriority(priority: "LoaderFirst" | "FileSystemFirst"): liquid.LiquidEngine
    /**
     * Sets whether the loader or filesystem is tried first for partials.
     *
     * The default is `LoaderFirst`.
     *
     * # Arguments
     *
     * * `this` - Engine reference returned for chaining.
     * * `priority` - Source to check first when both are configured.
     */
    setPartialPriority(priority: "LoaderFirst" | "FileSystemFirst"): liquid.LiquidEngine
    /**
     * Applies a plugin function to this engine and returns the engine.
     *
     * The plugin receives a reference to this engine and can register filters,
     * tags, blocks, or partial sources on it.
     *
     * # Arguments
     *
     * * `env` - JavaScript environment used to clone the engine reference.
     * * `this` - Engine reference passed to the plugin and returned afterward.
     * * `plugin` - Function that configures the engine.
     *
     * # Errors
     *
     * Returns an error if the engine reference cannot be cloned or the plugin
     * call fails.
     */
    use(plugin: LiquidPlugin): liquid.LiquidEngine
    /**
     * Renders a Liquid template using the supplied JSON context.
     *
     * The parser is built lazily and reused until registrations change.
     * Parsed templates are cached by template source. Rendering runs on a
     * blocking worker thread.
     *
     * # Arguments
     *
     * * `template_str` - Liquid template source.
     * * `context` - JSON object providing the template's variables.
     *
     * # Errors
     *
     * Returns an error if the parser or template cannot be built, the context
     * cannot be converted, rendering fails, a synchronization lock is
     * poisoned, or the worker thread panics.
     */
    render(templateStr: string, context: Record<string,any>): Promise<string>
  }
  /**
   * Parses template metadata and separates it from the Liquid body.
   *
   * The function first recognizes Jekyll-style YAML front matter beginning with
   * a standalone `---` line. It then recognizes a top-level
   * `{% layout "name" %}` tag. If neither form is present, the original source
   * is returned unchanged.
   *
   * Leading whitespace before the declaration is ignored for recognition, but
   * is not included in the parsed front matter or layout-tag body.
   *
   * # Arguments
   *
   * * `source` - Complete Liquid template source.
   *
   * # Errors
   *
   * Returns an N-API error when YAML front matter is invalid or is not a
   * key-value mapping.
   */
  export function frontMatter(source: string): liquid.ParsedFrontmatter
  /** File extensions supported when resolving templates from the filesystem. */
  export const enum LiquidExtension {
    /** The `html` extension. */
    Html = 'Html',
    /** The `liquid` extension. */
    Liquid = 'Liquid',
  }
  /**
   * Parsed template metadata and the remaining Liquid source.
   *
   * Front matter entries are available as a JSON object, with `layout` removed
   * and exposed separately through [`layout`](Self::layout).
   */
  export interface ParsedFrontmatter {
    /** Layout name declared in YAML front matter or by a top-level layout tag. */
    layout?: string
    /** Template source after the front-matter or layout declaration. */
    body: string
    /** Parsed YAML front-matter entries, excluding `layout`. */
    frontMatter: Record<string,any>
  }
  export const enum SourcePriority {
    /** Check the custom JS partial loader first, then fall back to disk. */
    LoaderFirst = 'LoaderFirst',
    /** Check the local file system directory first, then fall back to the JS loader. */
    FileSystemFirst = 'FileSystemFirst',
  }
  /**
   * Separates YAML-style front matter from a Liquid template source.
   *
   * The source has front matter only when it begins with a standalone `---`
   * line and contains a second standalone `---` line. Opening and closing
   * delimiters may use Unix or Windows line endings.
   *
   * # Arguments
   *
   * * `source` - Complete Liquid template source.
   *
   * # Returns
   *
   * A [`SplitTemplateResult`] containing the raw front matter and the remaining
   * body. If valid front matter is not found, `front_matter` is `None` and
   * `body` contains the original source.
   */
  export function splitFrontMatter(source: string): liquid.SplitTemplateResult
  /** The result of separating a Liquid template into front matter and body. */
  export interface SplitTemplateResult {
    /** Raw front matter, including both `---` delimiter lines, if present. */
    frontMatter?: string
    /** The template content after the closing front-matter delimiter. */
    body: string
  }
}

export declare namespace logger {
  /**
   * Writes a colored message to the console at the specified level.
   *
   * Informational and warning messages go to standard output; error messages
   * go to standard error.
   */
  export function log(level: logger.LogLevel, message: string): void
  /** Severity level for a console log message. */
  export const enum LogLevel {
    /** Informational message, written to standard output. */
    Info = 'info',
    /** Warning message, written to standard output. */
    Warning = 'warning',
    /** Error message, written to standard error. */
    Error = 'error',
  }
}

export declare namespace watch {
  /** Watches filesystem paths and sends debounced changes to a JavaScript callback. */
  export class FsWatcher {
    /**
     * Creates a watcher with a callback and optional configuration.
     *
     * The callback receives one [`WatchEvent`] per reported path. Invalid
     * ignore patterns and watcher initialization failures are returned as
     * errors.
     */
    constructor(callback: ((err: Error | null, arg: watch.WatchEvent) => void), options?: watch.WatchOptions | undefined | null)
    /**
     * Starts watching one path or a list of paths.
     *
     * Watching is recursive by default. Set `recursive` to `false` to watch
     * only the specified paths' immediate entries.
     */
    watch(targets: string | Array<string>, recursive?: boolean | undefined | null): void
    /** Stops watching one path or a list of paths. */
    unwatch(targets: string | Array<string>): void
    /** Stops the watcher and releases its callback. */
    close(): void
  }
  /** A filesystem change reported to the JavaScript callback. */
  export interface WatchEvent {
    /** Change type: `"create"`, `"modify"`, or `"remove"`. */
    kind: 'create' | 'modify' | 'remove'
    /** Path associated with the change. */
    path: string
  }
  /** Options for configuring an [`FsWatcher`]. */
  export interface WatchOptions {
    /** Glob patterns for paths to exclude from reported events. */
    ignore?: Array<string>
    /** Debounce window in milliseconds. Defaults to 100 ms. */
    debounceMs?: number
  }
}
