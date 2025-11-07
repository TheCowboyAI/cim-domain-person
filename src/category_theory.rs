//! Category Theory Traits
//!
//! Formal definitions of category theory concepts as Rust traits.
//! These provide mathematical rigor to our FRP implementation.
//!
//! # References
//! - Category Theory for Programmers (Bartosz Milewski)
//! - Haskell's Control.Monad
//! - The FRP-CT-COMPLIANCE.md document

/// Functor - Structure-preserving map between categories
///
/// A functor F maps:
/// - Objects: A → F(A)
/// - Morphisms: (A → B) → (F(A) → F(B))
///
/// # Laws
/// 1. **Identity**: `fmap(id) = id`
/// 2. **Composition**: `fmap(g ∘ f) = fmap(g) ∘ fmap(f)`
///
/// # Example
/// ```rust,ignore
/// use cim_domain_person::value_objects::PersonAttribute;
///
/// let attr = PersonAttribute::new(...);
/// let transformed = attr.fmap(|v| transform(v));
/// ```
pub trait Functor {
    /// The type being wrapped/contained
    type Inner;

    /// The output type after mapping
    type Output<B>;

    /// Map a function over the contained value
    ///
    /// This must preserve structure - only the value changes,
    /// not the container shape.
    fn fmap<F, B>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Inner) -> B;
}

/// Monad - Composition of computational effects
///
/// A monad provides:
/// - `pure`: Inject a pure value into the monad
/// - `bind` (>>=): Chain computations that produce monadic values
///
/// # Laws
/// 1. **Left Identity**: `pure(a).bind(f) = f(a)`
/// 2. **Right Identity**: `m.bind(pure) = m`
/// 3. **Associativity**: `m.bind(f).bind(g) = m.bind(|x| f(x).bind(g))`
///
/// # Example
/// ```rust,ignore
/// use cim_domain_person::value_objects::PersonAttributeSet;
///
/// let result = PersonAttributeSet::pure(attr)
///     .bind(|a| validate(a))
///     .bind(|a| enrich(a))
///     .bind(|a| transform(a));
/// ```
pub trait Monad: Functor {
    /// Inject a pure value into the monad
    ///
    /// Also known as `return` in Haskell
    fn pure<A>(value: A) -> Self
    where
        Self: Sized;

    /// Monadic bind - chain computations
    ///
    /// Also known as `>>=` (bind operator) in Haskell
    fn bind<F, B>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Inner) -> Self::Output<B>;

    /// Sequence two monadic actions, discarding the first result
    ///
    /// `m1.then(m2)` executes m1, then m2, keeping only m2's result
    fn then<B>(self, next: Self::Output<B>) -> Self::Output<B>
    where
        Self: Sized,
        Self::Output<B>: Monad<Inner = B>,
    {
        self.bind(|_| next)
    }
}

/// Applicative Functor - Apply wrapped functions to wrapped values
///
/// Sits between Functor and Monad in the hierarchy.
/// Provides function application within a context.
///
/// # Laws
/// 1. **Identity**: `pure(id).apply(v) = v`
/// 2. **Composition**: `pure(compose).apply(u).apply(v).apply(w) = u.apply(v.apply(w))`
/// 3. **Homomorphism**: `pure(f).apply(pure(x)) = pure(f(x))`
/// 4. **Interchange**: `u.apply(pure(y)) = pure(|f| f(y)).apply(u)`
pub trait Applicative: Functor {
    /// Apply a wrapped function to a wrapped value
    fn apply<F, B>(self, f: Self::Output<F>) -> Self::Output<B>
    where
        F: FnOnce(Self::Inner) -> B;
}

/// Coalgebra - Unfold from state to observable structure
///
/// Dual of algebra. While algebras fold/reduce:
///   F(A) → A
///
/// Coalgebras unfold/expand:
///   A → F(A)
///
/// # Example
/// A Person (state) unfolds into PersonAttributeSet (observable structure)
pub trait Coalgebra {
    /// The functor type that represents observable structure
    type Observation;

    /// Unfold the internal state into observable structure
    ///
    /// This is the coalgebra morphism: State → F(State)
    fn unfold(&self) -> Self::Observation;
}

/// Natural Transformation - Functor-to-functor mapping
///
/// A natural transformation η: F → G transforms one functor into another
/// while preserving structure.
///
/// # Law
/// For any f: A → B:
/// ```text
/// G(f) ∘ η_A = η_B ∘ F(f)
/// ```
///
/// This is the "naturality condition" - the transformation commutes with fmap.
///
/// # Example
/// Transform Person domain to Healthcare domain while preserving structure
pub trait NaturalTransformation<F, G>
where
    F: Functor,
    G: Functor,
{
    /// Apply the natural transformation
    ///
    /// Converts F(A) to G(A) for any type A
    fn transform<A>(source: F) -> G
    where
        F: Functor<Inner = A>,
        G: Functor<Inner = A>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example implementation for Option (which is a Monad)
    impl<T> Functor for Option<T> {
        type Inner = T;
        type Output<B> = Option<B>;

        fn fmap<F, B>(self, f: F) -> Self::Output<B>
        where
            F: FnOnce(Self::Inner) -> B,
        {
            self.map(f)
        }
    }

    #[test]
    fn test_functor_identity_law() {
        let value = Some(42);
        let identity = |x| x;

        let result = value.fmap(identity);

        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_functor_composition_law() {
        let value = Some(10);
        let f = |x| x + 1;
        let g = |x| x * 2;

        // fmap(g ∘ f)
        let composed = value.clone().fmap(|x| g(f(x)));

        // fmap(g) ∘ fmap(f)
        let separate = value.fmap(f).fmap(g);

        assert_eq!(composed, separate);
    }

    #[test]
    fn test_option_as_functor() {
        let some_value: Option<i32> = Some(5);
        let none_value: Option<i32> = None;

        let doubled_some = some_value.fmap(|x| x * 2);
        let doubled_none = none_value.fmap(|x| x * 2);

        assert_eq!(doubled_some, Some(10));
        assert_eq!(doubled_none, None);
    }
}
