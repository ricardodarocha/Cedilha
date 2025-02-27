use std::collections::{VecDeque, HashMap, HashSet};
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(name = "BooleanLang")]
struct Args {
    #[arg(long = "exp", help = "Expressão booleana a ser avaliada")]
    expression: Option<String>,
    
    #[arg(short = 's', long = "silence", help = "Modo silencioso, sem imprimir a tabela verdade")]
    s: bool,
    
    #[arg(short = 'e', long = "expand", help = "Expande os cálculos intermediários")]
    e: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    And,
    Or,
    Not,
    LParen,
    RParen,
    Var(String),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let words: Vec<&str> = input.split_whitespace().collect();
    
    for word in words {
        match word {
            "and" => tokens.push(Token::And),
            "or" => tokens.push(Token::Or),
            "not" => tokens.push(Token::Not),
            "(" => tokens.push(Token::LParen),
            ")" => tokens.push(Token::RParen),
            var => tokens.push(Token::Var(var.to_string())),
        }
    }
    
    tokens
}

#[derive(Debug)]
enum AST {
    And(Box<AST>, Box<AST>),
    Or(Box<AST>, Box<AST>),
    Not(Box<AST>),
    Var(String),
}

fn parse_expr(tokens: &mut VecDeque<Token>) -> Option<AST> {
    let mut stack: Vec<AST> = Vec::new();
    let mut operators: Vec<Token> = Vec::new();

    while let Some(token) = tokens.pop_front() {
        match token {
            Token::Var(v) => stack.push(AST::Var(v)),
            Token::Not => operators.push(Token::Not),
            Token::LParen => {
                if let Some(expr) = parse_expr(tokens) {
                    stack.push(expr);
                }
            }
            Token::RParen => break,
            Token::And | Token::Or => {
                while let Some(op) = operators.pop() {
                    if let Some(right) = stack.pop() {
                        if let Some(left) = stack.pop() {
                            let node = match op {
                                Token::And => AST::And(Box::new(left), Box::new(right)),
                                Token::Or => AST::Or(Box::new(left), Box::new(right)),
                                _ => unreachable!(),
                            };
                            stack.push(node);
                        } else {
                            stack.push(right);
                        }
                    }
                }
                operators.push(token);
            }
        }
    }

    while let Some(op) = operators.pop() {
        if let Some(right) = stack.pop() {
            if let Some(left) = stack.pop() {
                let node = match op {
                    Token::And => AST::And(Box::new(left), Box::new(right)),
                    Token::Or => AST::Or(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
                stack.push(node);
            } else {
                stack.push(right);
            }
        }
    }

    stack.pop()
}

fn extract_variables(ast: &AST, vars: &mut HashSet<String>) {
    match ast {
        AST::Var(v) => { vars.insert(v.clone()); },
        AST::And(left, right) | AST::Or(left, right) => {
            extract_variables(left, vars);
            extract_variables(right, vars);
        }
        AST::Not(expr) => extract_variables(expr, vars),
    }
}

fn evaluate_steps(ast: &AST, values: &HashMap<String, bool>, steps: &mut HashMap<String, bool>) -> bool {
    match ast {
        AST::Var(v) => *values.get(v).unwrap_or(&false),
        AST::And(left, right) => {
            let l = evaluate_steps(left, values, steps);
            let r = evaluate_steps(right, values, steps);
            let result = l && r;
            steps.insert(format!("({:?} and {:?})", left, right), result);
            result
        }
        AST::Or(left, right) => {
            let l = evaluate_steps(left, values, steps);
            let r = evaluate_steps(right, values, steps);
            let result = l || r;
            steps.insert(format!("({:?} or {:?})", left, right), result);
            result
        }
        AST::Not(expr) => {
            let val = evaluate_steps(expr, values, steps);
            let result = !val;
            steps.insert(format!("not {:?}", expr), result);
            result
        }
    }
}

fn generate_truth_table(ast: &AST) {
    let mut vars = HashSet::new();
    extract_variables(ast, &mut vars);
    let vars: Vec<String> = vars.into_iter().collect();
    let total_rows = 1 << vars.len();
    
    println!("\nTabela Verdade:");
    println!("{:?} => Resultado", vars);
    
    for i in 0..total_rows {
        let mut values = HashMap::new();
        let mut steps = HashMap::new();
        
        for (j, var) in vars.iter().enumerate() {
            values.insert(var.clone(), (i & (1 << j)) != 0);
        }
        let result = evaluate_steps(ast, &values, &mut steps);
        
        println!("{:?} => {}", values, result);
        
        println!("Passos do cálculo:");
        for (step, res) in &steps {
            println!("{} = {}", step, res);
        }
        println!("-------------------");
    }
}

fn main() {
    let args = Args::parse();
    
    let input = args.expression.unwrap_or_else(|| "a and (b or not c)".to_string());
    let tokens = tokenize(&input);
    let mut token_queue = VecDeque::from(tokens);
    
    if let Some(ast) = parse_expr(&mut token_queue) {
        println!("AST: {:?}", ast);
        if !args.s {
            generate_truth_table(&ast);
        }
    }
      }
  
