use clap::ValueEnum;

/// Stageies to handle a dict in an array
/// Example:
/// ```json
/// {
///  "a" : [1,2,3]
/// }
/// ```
///
/// How to add "a.b" : 4 ?
#[derive(Default, Debug, ValueEnum, Clone)]
pub enum HowToDictInArray {
    /// Will generate an error when adding the line
    #[default]
    GenerateError,
    /// Will merge the dict in the array, ie the array will be:
    /// ```json
    /// {
    /// "a" : [1,2,3, {"b": 4}]
    /// }
    /// ```
    MergeDictInArray,

    /// Will create a new array with the dict as value, ie the array will be:
    /// ```json
    /// {
    /// "a" : {"array": [1,2,3],   "b": 4}
    /// }
    /// ```
    MakeArrayAsDictValue,
}

#[derive(Debug, Default)]
pub struct EngineOptions {
    pub verbosity: u8,
    pub how_to_dict_in_array: HowToDictInArray,
}

impl EngineOptions {
    pub fn new() -> Self {
        Self {
            verbosity: 0,
            ..Default::default()
        }
    }

    pub fn with_verbosity(mut self, verbosity: u8) -> Self {
        self.verbosity = verbosity;
        self
    }



    pub fn with_how_to_dict_in_array(mut self, h: HowToDictInArray) -> Self {
        self.how_to_dict_in_array = h;
        self
    }
}
