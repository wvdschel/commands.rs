//! # Parser Nodes
//!
//! Parser nodes represent the grammar that is defined
//! by the currently permissible set of commands and their
//! parameters.

use std::rc::Rc;

/// Minimum priority.
pub const PRIORITY_MINIMUM: i32 = -10000;
/// The default priority for a parameter.
pub const PRIORITY_PARAMETER: i32 = -10;
/// The default priority.
pub const PRIORITY_DEFAULT: i32 = 0;

/// A node in the tree of commands and their parameters
/// used by the `Parser`.
pub trait Node {
    /// The data describing this node.
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields;

    /// Nodes that are children of this node. Used to
    /// by the `Parser` during `advance`, `complete`, etc.
    fn successors(&self) -> &Vec<Rc<Node>> {
        &self.node_data().successors
    }

    /// The text used to identify this node in help text.
    /// This is typically the node name, either in plain
    /// form or decorated for parameters.
    fn help_symbol(&self) -> String {
        self.node_data().name.to_string()
    }

    /// Help text describing this node.
    fn help_text(&self) -> &Option<String> {
        unimplemented!();
    }

    /// Hidden nodes are still found for matching, but are
    /// hidden from completion.
    fn hidden(&self) -> bool {
        self.node_data().hidden
    }

    /// The name of this node.
    fn name(&self) -> &String {
        &self.node_data().name
    }

    /// This priority of this node during matching and completion.
    fn priority(&self) -> i32 {
        self.node_data().priority
    }
}

impl PartialEq for Node {
    /// Nodes are equal based on pointer equality.
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

/// A parse tree node.
#[doc(hidden)]
pub struct NodeFields {
    /// Possible successor nodes. Collected while building.
    successors: Vec<Rc<Node>>,
    /// The name of this node.
    name: String,
    /// Match and complete priority.
    priority: i32,
    /// Hidden nodes are not completed. This doesn't modify matching.
    hidden: bool,
}

/// The root of a command tree.
///
/// ```
/// use commands::parser::nodes::RootNode;
///
/// let root = RootNode::new();
/// ```
pub struct RootNode {
    node_fields: NodeFields,
}

impl RootNode {
    /// Create a new `RootNode`
    pub fn new() -> Rc<Self> {
        Rc::new(RootNode {
            node_fields: NodeFields {
                successors: vec![],
                name: "__root__".to_string(),
                priority: PRIORITY_DEFAULT,
                hidden: false,
            },
        })
    }
}

impl Node for RootNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

/// A node representing a command.
pub struct CommandNode {
    node_fields: NodeFields,
    command_fields: CommandNodeFields,
}

struct CommandNodeFields {
    help: Option<String>,
    handler: Option<fn(&node: Node) -> ()>,
    parameters: Vec<Rc<ParameterNode>>,
}

impl CommandNode {
    /// Construct a new `CommandNode`.
    pub fn new(name: &str,
               priority: i32,
               hidden: bool,
               help: Option<String>,
               handler: Option<fn(&node: Node) -> ()>)
               -> Rc<Self> {
        Rc::new(CommandNode {
            node_fields: NodeFields {
                successors: vec![],
                name: name.to_string(),
                priority: priority,
                hidden: hidden,
            },
            command_fields: CommandNodeFields {
                help: help,
                handler: handler,
                parameters: vec![],
            },
        })
    }
}
impl Node for CommandNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }

    fn help_text(&self) -> &Option<String> {
        &self.command_fields.help
    }
}

impl CommandNode {
    /// The handler which is executed once this node has been accepted.
    pub fn handler(&self) -> Option<fn(&node: Node) -> ()> {
        self.command_fields.handler
    }

    /// Get the parameter nodes for this command.
    pub fn parameters(&self) -> &Vec<Rc<ParameterNode>> {
        &self.command_fields.parameters
    }
}

/// A wrapper node wraps another command.
///
/// This is used for the help command so that it can complete
/// normal commands.
///
/// The `successors` will be those of the wrapped node.
pub struct WrapperNode {
    node_fields: NodeFields,
    #[allow(dead_code)]
    command_fields: CommandNodeFields,
    root: Rc<Node>,
}

impl Node for WrapperNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }

    fn successors(&self) -> &Vec<Rc<Node>> {
        self.root.successors()
    }
}

/// A repeatable node is an internal helper for representing
/// nodes that can be repeated, like some parameters.
pub trait RepeatableNode: Node {
    /// Internal data for a repeatable node.
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields;

    /// Whether or not this node can be repeated. A repeated
    /// node can be `accept`ed multiple times.
    fn repeatable(&self) -> bool {
        self.repeatable_data().repeatable
    }

    /// If present, this node will no longer be `acceptable`.
    fn repeat_marker(&self) -> &Option<Rc<Node>> {
        &self.repeatable_data().repeat_marker
    }
}

/// The data for a repeatable node.
#[doc(hidden)]
pub struct RepeatableNodeFields {
    repeatable: bool,
    repeat_marker: Option<Rc<Node>>,
}

/// A node that represented the name portion of a named
/// parameter.
pub struct ParameterNameNode {
    node_fields: NodeFields,
    repeatable_fields: RepeatableNodeFields,
    help: Option<String>,
    parameter: Rc<Node>,
}

impl Node for ParameterNameNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }

    fn help_symbol(&self) -> String {
        self.node_fields.name.clone() + " " + self.parameter.help_symbol().as_str()
    }

    fn help_text(&self) -> &Option<String> {
        &self.help
    }
}

impl RepeatableNode for ParameterNameNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.repeatable_fields
    }
}

/// Parameter nodes.
pub trait ParameterNode {
    /// Internal data for a parameter node.
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields;

    /// A `required` parameter must be supplied for the
    /// command line being parsed to be valid.
    fn required(&self) -> bool {
        self.parameter_data().required
    }
}

impl RepeatableNode for ParameterNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.parameter_data().repeatable
    }
}

/// Data for parameter nodes.
#[doc(hidden)]
pub struct ParameterNodeFields {
    node: NodeFields,
    repeatable: RepeatableNodeFields,
    help: Option<String>,
    required: bool,
}

impl Node for ParameterNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.parameter_data().node
    }

    fn help_symbol(&self) -> String {
        String::from("<") + self.node_data().name.as_str() +
        if self.repeatable() {
            ">..."
        } else {
            ">"
        }
    }

    fn help_text(&self) -> &Option<String> {
        &self.parameter_data().help
    }
}

/// A flag parameter node.
///
/// When implemented, this will only have a value of
/// true when it is present.
pub struct FlagParameterNode {
    parameter_fields: ParameterNodeFields,
}

impl ParameterNode for FlagParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}

/// A named parameter node.
pub struct NamedParameterNode {
    parameter_fields: ParameterNodeFields,
}

impl ParameterNode for NamedParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}

/// A simple parameter node. This is only present in a command
/// line as a value.
pub struct SimpleParameterNode {
    parameter_fields: ParameterNodeFields,
}

impl ParameterNode for SimpleParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}
