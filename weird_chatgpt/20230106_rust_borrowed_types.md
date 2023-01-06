# Rust Borrowed Types

**January 6, 2023**

This chat was inspired by this article about [using borrowed types in function arguments](https://rust-unofficial.github.io/patterns/idioms/coercion-arguments.html) and this Stack Overflow answer about [the same topic](https://stackoverflow.com/questions/40006219/why-is-it-discouraged-to-accept-a-reference-to-a-string-string-vec-vec-o).

This was an interesting chat and an example on how I leveraged the statistical model of ChatGPT to confirm my understanding of a new topic.

#### Context:

I read the article months ago, but I didn't had first-hand experience with Rust at that time.

After I started experimenting with the language a couple weeks ago, I found myself using a lot of `&String` and `&Vec` as function arguments, and it caused a set of code smells outside of it.

Suddenly, the article was relatable, and I went to re-read it. I also found the Stack Overflow answer, which was a great complement to the article.

Then, I went to ChatGPT to check how well it could explain the topic and reveal missing spots.

This chat was the result:

---

#### I read that it's not recommended to accept `&String` or `&Vec` arguments in Rust directly. Why? And what should I use instead?

It is generally not recommended to accept a reference to a String or Vec as a function argument in Rust because these types own their own memory and have a known capacity. This means that if you accept a reference to one of these types, you do not have control over when the memory gets deallocated or how it is resized.

Instead, it is generally better to accept one of the following as a function argument: - A reference to a slice `&[T]`: This allows you to borrow a portion of a collection without taking ownership of the whole thing. - A generic type parameter `T` that has the `AsRef<[T]>` trait bound: This allows the function to accept a variety of types that can be converted into a slice, such as arrays, vectors, and even other slices.

For example, here is a function that takes a slice of `i32` values and returns the sum of all the values in the slice:

```rust
fn sum(values: &[i32]) -> i32 {
    let mut total = 0;
    for value in values {
        total += value;
    }
    total
}
```

And here is a function that takes a generic type `T` and returns the sum of all the values in a slice that can be created from the type:

```rust
fn sum<T: AsRef<[i32]>>(values: T) -> i32 {
    let mut total = 0;
    for value in values.as_ref() {
        total += value;
    }
    total
}
```

---

#### Does it extend to struct fields as well? Is it generally recommended to have Owned Types or slices as the type of struct fields?

It is generally more recommended to use owned types as the fields of a struct rather than references to slices.

This is because structs are meant to own and manage their own data, and using references as fields can make it more difficult to understand and reason about the ownership and lifetime of the data within the struct.

Additionally, using owned types as fields allows you to implement methods on the struct that can mutate the fields in place, which can be more convenient than having to pass in references to slices every time you want to update the data.

That being said, there are certainly cases where it is appropriate to use references to slices as fields of a struct. For example, if the data is very large and you want to avoid copying it, or if the data is borrowed from another source and you do not want to take ownership of it. In these cases, it is important to carefully consider the trade-offs and make sure that the lifetime of the reference is well-defined.

---

#### How a method acting in one of the owned fields would look like? If the field is an owned type such as String or Vec, which argument the method should accept?

Here is an example of a struct with an owned `Vec` field and a method that adds an element to the end of the vector:

```rust
struct MyStruct {
    data: Vec<i32>,
}

impl MyStruct {
    fn add(&mut self, value: i32) {
        self.data.push(value);
    }
}
```

The method takes a mutable reference to self as its first argument and accepts the value to be added as a separate argument.

Here is an example of how you might use this method:

```rust
let mut s = MyStruct { data: vec![1, 2, 3] };
s.add(4);
assert_eq!(s.data, [1, 2, 3, 4]);
```

Note that the method has a mutable reference to self because it needs to modify the field data in place.

If the field was a `String` instead of a `Vec`, the method could look like this:

```rust
struct MyStruct {
    data: String,
}

impl MyStruct {
    fn add(&mut self, value: &str) {
        self.data.push_str(value);
    }
}
```

In this case, the method takes a reference to a string slice `&str` as its argument, because the `String` type has a method `push_str` that takes a string slice as an argument.

---

#### The string example is interesting and it's what I was looking for. Notice that the data in the `MyStruct` is a `String` while the method argument that changes it is a `&str`. Is this the recommended pattern for general modeling? What if I want to change the data directly and it's an static method that does not take `self` as the first argument?

Yes, it is generally a good pattern to use a string slice `&str` as the argument to a method that modifies a `String` field, because it allows the method to accept a variety of types that can be converted into a string slice, such as `&str` itself, `String`, and even other `String` types.

If you want to change the data directly and the method is a static method that does not take self as the first argument, you can simply pass in a `String` or `&mut String` as the argument instead.

Here is an example of a static method that takes a `String` as an argument and appends a string slice to it:

```rust
struct MyStruct {
    data: String,
}

impl MyStruct {
    fn add(data: &mut String, value: &str) {
        data.push_str(value);
    }
}
```

Here is an example of how you might use this method:

```rust
let mut s = MyStruct { data: "Hello".to_string() };
MyStruct::add(&mut s.data, ", world!");
assert_eq!(s.data, "Hello, world!");
```

Keep in mind that static methods do not have access to the fields of a struct, so you will need to pass in the field as an argument if you want to modify it.

---

#### What if it's a constructor?

If you want to create a constructor for a struct that takes a String as an argument and sets it as the value of a field, you can simply define a function that takes a `String` as an argument and returns an instance of the struct with the field set to the value of the argument.

Here is an example of a struct with a `String` field and a constructor function that sets the value of the field:

```rust
struct MyStruct {
    data: String,
}

impl MyStruct {
    fn new(data: String) -> MyStruct {
        MyStruct { data }
    }
}
```

Here is an example of how you might use this constructor:

```rust
let s = MyStruct::new("Hello".to_string());
assert_eq!(s.data, "Hello");
```

Alternatively, you can also define a constructor as a function that takes a string slice `&str` as an argument and creates a String from it internally:

```rust
struct MyStruct {
    data: String,
}

impl MyStruct {
    fn new(data: &str) -> MyStruct {
        MyStruct { data: data.to_string() }
    }
}
```

This allows you to use the constructor with a variety of types that can be converted into a string slice, such as `&str` itself and String.

Here is an example of how you might use this constructor:

```rust
let s = MyStruct::new("Hello");
assert_eq!(s.data, "Hello");
```

---

#### Let's try a more complex example with a struct containing multiple owned types, and the arguments for the methods accepting the underlying type reference. Explain why it's a good practice, or why not.

Sure! Here is an example of a struct with multiple owned fields and methods that take references to the underlying types as arguments:

```rust
struct MyStruct {
    data1: Vec<i32>,
    data2: String,
}

impl MyStruct {
    fn add_to_vec(&mut self, value: &i32) {
        self.data1.push(*value);
    }

    fn add_to_string(&mut self, value: &str) {
        self.data2.push_str(value);
    }
}
```

Using references to the underlying types as arguments to these methods is a good practice because it allows you to accept a variety of types that can be converted into the appropriate reference type.

For example, the add_to_vec method can accept a reference to an `i32` value, while the add_to_string method can accept a string slice `&str`. This makes the methods more flexible and allows them to be used with a wider range of types.

Additionally, using references as arguments allows you to modify the fields of the struct in place without taking ownership of the data, which can be more convenient than having to return a new instance of the struct with the modified data.

Keep in mind that using references as arguments does not necessarily mean that the data will not be modified. If you want to ensure that the data is not modified, you can use immutable references `&T` instead of mutable references `&mut T`.
