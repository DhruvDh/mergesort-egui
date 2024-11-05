---
title: "The Journey to Merge Sort: Part 1"
---

## Starting with Regular Sort

Let's begin with the most intuitive way to sort numbers. Imagine you have a sequence of numbers:

```
7  2  4  1  5  3
```

How would you sort this by hand? Most people would:

1. Look at each number
2. Find where it belongs in the sequence we've sorted so far
3. Insert it there

Let's implement this natural approach:

```java
public class Sorting {
    public static void regularSort(int[] arr) {
        // For each number (except the first one)
        for (int i = 1; i < arr.length; i++) {
            int current = arr[i];  // The number we're placing
            int j = i - 1;         // Look at previous numbers
            
            // Move larger numbers one position ahead
            while (j >= 0 && arr[j] > current) {
                arr[j + 1] = arr[j];
                j--;
            }
            
            // Place our number in the gap we created
            arr[j + 1] = current;
        }
    }

    public static void main(String[] args) {
        int[] numbers = {7, 2, 4, 1, 5, 3};
        
        System.out.println("Before sorting: " + Arrays.toString(numbers));
        regularSort(numbers);
        System.out.println("After sorting:  " + Arrays.toString(numbers));
    }
}
```

Let's trace what happens with our example:

```
Initial:    [7, 2, 4, 1, 5, 3]
Place 2:    [2, 7, 4, 1, 5, 3]    // 2 moves before 7
Place 4:    [2, 4, 7, 1, 5, 3]    // 4 moves before 7
Place 1:    [1, 2, 4, 7, 5, 3]    // 1 moves to start
Place 5:    [1, 2, 4, 5, 7, 3]    // 5 moves before 7
Place 3:    [1, 2, 3, 4, 5, 7]    // 3 moves after 2
```

This works! But let's think about how much work we're doing. For each number:

- We might need to shift ALL previous numbers up one position
- The further right we go, the more numbers we might need to shift

Let's look at a worst-case scenario:

```
Initial:    [5, 4, 3, 2, 1]
Place 4:    [4, 5, 3, 2, 1]    // 1 shift
Place 3:    [3, 4, 5, 2, 1]    // 2 shifts
Place 2:    [2, 3, 4, 5, 1]    // 3 shifts
Place 1:    [1, 2, 3, 4, 5]    // 4 shifts
```

For an array of size n:

- First number: up to 1 shift
- Second number: up to 2 shifts
- Third number: up to 3 shifts
...and so on.

This means we're doing 1 + 2 + 3 + ... + (n-1) operations in the worst case.
This sum equals (n-1)(n)/2, which means our algorithm takes O(n²) time in the worst case.

Can we do better?

Remember how we made binary search faster - by eliminating half the search space at each step. Could we use a similar idea here?

Well, not exactly the same way... In binary search, we could completely ignore one half of the array because we knew our target wasn't there. With sorting, we can't just ignore half the numbers - they all need to end up in the right place!

But what if, instead of ignoring half the numbers, we sorted them separately? Let's think about what that might look like:

```
Original:     [7, 2, 4, 1, 5, 3]
Split:        [7, 2, 4] [1, 5, 3]
```

If we sort each half using our regular sort, how many operations would that take?

- Left half (size n/2): up to (n/2-1)(n/2)/2 operations
- Right half (size n/2): up to (n/2-1)(n/2)/2 operations

Let's work out the math:

- Original algorithm: about n²/2 operations
- With splitting: about n²/4 operations

Hey - that's interesting! It's still O(n²), but we've cut the number of operations roughly in half!

Should we try implementing this idea? Even if it doesn't give us a fundamentally faster algorithm, it might teach us something...

---

title: "The Journey to Merge Sort: Part 2"
---

## Trying the Split Approach

Since splitting the array might save us some work, let's implement this idea:

```java
public static void splitSort(int[] arr) {
    // Base case: arrays of size 0 or 1 are already sorted
    if (arr.length <= 1) return;
    
    // Split array in half
    int mid = arr.length / 2;
    int[] left = Arrays.copyOfRange(arr, 0, mid);
    int[] right = Arrays.copyOfRange(arr, mid, arr.length);
    
    // Sort each half using our regular sort
    regularSort(left);
    regularSort(right);
    
    // Put the pieces back together
    System.arraycopy(left, 0, arr, 0, left.length);
    System.arraycopy(right, 0, arr, left.length, right.length);
}
```

Let's try it with our example:

```java
public static void main(String[] args) {
    int[] numbers = {7, 2, 4, 1, 5, 3};
    System.out.println("Original array: " + Arrays.toString(numbers));
    
    splitSort(numbers);
    System.out.println("After splitSort: " + Arrays.toString(numbers));
}
```

Running this gives us:

```
Original array: [7, 2, 4, 1, 5, 3]
After splitSort: [2, 4, 7, 1, 3, 5]
```

Wait a minute... that's not right! The numbers are sorted within each half, but when we put them back together, something's wrong. Let's look at what happened step by step:

1. Split:        [7, 2, 4] | [1, 5, 3]
2. Sort left:    [2, 4, 7] | [1, 5, 3]
3. Sort right:   [2, 4, 7] | [1, 3, 5]
4. Combined:     [2, 4, 7, 1, 3, 5]

Ah! Even though each half is sorted, 7 is greater than 1, so just putting the sorted halves next to each other doesn't work.

Before we continue, take a moment to think:

- What information do we have that we're not using?
- Is there something special about these two halves that could help us?
- If you had to combine these by hand, how would you do it?

Try working through combining [2, 4, 7] and [1, 3, 5] by hand. What strategy would you use?

[Space for thinking...]

Let's explore some possible strategies:

1. What if we took alternating elements?
   [2, 4, 7] and [1, 3, 5] -> [2, 1, 4, 3, 7, 5]
   No, that's still not sorted...

2. What if we took the smallest element first?
   Looking at [2, 4, 7] and [1, 3, 5]
   - 1 is the smallest overall, so take it
   Now looking at [2, 4, 7] and [3, 5]
   - 2 is the smallest remaining...

Hey, that second idea seems promising! Let's work through it completely:

[Space for working it out...]

Would you like to try working through the complete combination before we continue?

## Discovering the Merge

We found that taking the smallest element at each step might work. Let's explore this idea more carefully.

Given our two sorted halves: [2, 4, 7] and [1, 3, 5]

Step 1: We need the smallest element overall

- Left half starts with 2
- Right half starts with 1
- 1 is smaller, so it goes first
- Now we have: [1] and need to merge [2, 4, 7] with [3, 5]

Step 2: Looking at the fronts again

- Left half starts with 2
- Right half starts with 3
- 2 is smaller, so it goes next
- Now we have: [1, 2] and need to merge [4, 7] with [3, 5]

Do you see a pattern forming? At each step, we:

1. Look at the front of both remaining sequences
2. Take the smaller one
3. Move forward in whichever pile we took from

Before we try coding this, what information do we need to keep track of? Take a moment to list out the important pieces...

[Space for thinking...]

We probably need:

1. Our position in the left half (which number are we looking at?)
2. Our position in the right half
3. Where we're putting numbers in our result

Let's start with just the first comparison. How would we write that?

```java
// If we're looking at position leftPos in left half
// and position rightPos in right half...
if (left[leftPos] <= right[rightPos]) {
    // The left number is smaller (or equal)
    // What should we do here?
} else {
    // The right number is smaller
    // What should we do here?
}
```

Can you fill in what should happen in each case? Remember, we need to:

- Take the smaller number
- Put it somewhere
- Move forward in the correct half

[Space for thinking...]

Let's complete one case:

```java
if (left[leftPos] <= right[rightPos]) {
    // Take the left number
    arr[arrPos] = left[leftPos];
    // Move forward in left half
    leftPos++;
    // Move forward in result
    arrPos++;
} else {
    // Can you fill in this part?
}
```

Try completing the else case yourself before continuing...

[Space for coding...]

Now we can do one comparison, but we need to keep doing this until... when? Think about:

- When can we keep comparing numbers from both halves?
- What happens when we run out of numbers in one half?

[Space for thinking...]

We need to keep going while we have numbers in both halves:

```java
while (leftPos < left.length && rightPos < right.length) {
    // Our comparison code here
}
```

But what happens when one half is empty? Consider this case:

- Left half:  [2, 4, 7]
- Right half: [1, 3]
After some merging:
- Result so far: [1, 2, 3]
- Left half remaining: [4, 7]
- Right half remaining: []

What should we do with the remaining numbers?

[Space for thinking...]

Since each half was sorted to begin with, if we have leftovers in one half, they must all be larger than what we've placed so far. We can just copy them over in order!

Would you like to try writing the complete merge code now? Here's a template to start with:

```java
public static void splitAndMergeSort(int[] arr) {
    if (arr.length <= 1) return;
    
    // Split and sort as before
    int mid = arr.length / 2;
    int[] left = Arrays.copyOfRange(arr, 0, mid);
    int[] right = Arrays.copyOfRange(arr, mid, arr.length);
    
    regularSort(left);
    regularSort(right);
    
    // Merging part:
    int leftPos = 0;
    int rightPos = 0;
    int arrPos = 0;
    
    // Your code here!
    // 1. While we have elements in both halves...
    // 2. Handle any remaining elements...
}
```

Try completing this implementation yourself!

## A Curious Observation

Now that we have a working splitAndMergeSort, let's look at it again:

```java
public static void splitAndMergeSort(int[] arr) {
    if (arr.length <= 1) return;
    
    // 1. Split
    int mid = arr.length / 2;
    int[] left = Arrays.copyOfRange(arr, 0, mid);
    int[] right = Arrays.copyOfRange(arr, mid, arr.length);
    
    // 2. Sort each half
    regularSort(left);
    regularSort(right);
    
    // 3. Merge the sorted halves
    // (our merge code from before)
}
```

Look at what we're doing with each half:

1. Take an array
2. Sort it
3. Ensure it's ready for merging

But wait... isn't that exactly what our splitAndMergeSort method does? We're using regularSort, but why? Take a moment to think about this...

[Space for thinking...]

Let's trace what would happen if we used splitAndMergeSort recursively. Take this example:

```
[7, 2, 4, 1, 5, 3]
```

Try drawing out what would happen if splitAndMergeSort called itself instead of regularSort. Start with just the splitting part:

[Space for drawing the recursion tree...]

Here's the first split:

```
         [7, 2, 4, 1, 5, 3]
         /              \
   [7, 2, 4]         [1, 5, 3]
```

What would happen to [7, 2, 4]? It would get split again:

```
         [7, 2, 4, 1, 5, 3]
         /              \
   [7, 2, 4]         [1, 5, 3]
   /        \
[7]      [2, 4]
```

Can you continue this tree? Draw out all the splits before reading further.

[Space for completing the tree...]

Eventually we get:

```
              [7, 2, 4, 1, 5, 3]
              /              \
        [7, 2, 4]         [1, 5, 3]
        /        \         /       \
     [7]      [2, 4]    [1]    [5, 3]
              /    \           /     \
            [2]    [4]       [5]    [3]
```

Now, starting from the bottom:

1. Merge [2] and [4] -> [2, 4]
2. Merge [2, 4] with [7] -> [2, 4, 7]
3. Merge [5] and [3] -> [3, 5]
4. Merge [3, 5] with [1] -> [1, 3, 5]
5. Finally merge [2, 4, 7] and [1, 3, 5] -> [1, 2, 3, 4, 5, 7]

Try tracing through this with a small example of your own before continuing...

[Space for trying your own example...]

Should we try modifying our code to use this recursive approach? Here's a template:

```java
public static void splitAndMergeSort(int[] arr) {
    if (arr.length <= 1) return;
    
    int mid = arr.length / 2;
    int[] left = Arrays.copyOfRange(arr, 0, mid);
    int[] right = Arrays.copyOfRange(arr, mid, arr.length);
    
    // What should go here instead of regularSort?
    
    // Merge code stays the same
}
```

Try completing this implementation yourself!

## Understanding the Efficiency

Before we analyze our complete algorithm, let's understand how much work each part does. First, let's look at merging.

### The Merge Step

Remember our merge process? We look at the front of both arrays and pick the smaller element each time. Let's count the steps for merging [2, 4, 7] and [1, 3, 5]:

```
Step 1: Compare 2 and 1
        Take 1
        Arrays: [2, 4, 7] and [3, 5]
        Result: [1]

Step 2: Compare 2 and 3
        Take 2
        Arrays: [4, 7] and [3, 5]
        Result: [1, 2]

Step 3: Compare 4 and 3
        Take 3
        Arrays: [4, 7] and [5]
        Result: [1, 2, 3]
```

...and so on.

How many steps do we take? Each step puts one number in its final position, and we need to place all numbers. So if we're merging arrays with a total of N numbers, we do N steps.

Try counting steps yourself for merging [1, 3, 5] and [2, 4, 6, 8]. How many steps?

[Space for counting...]

### The Complete Picture

Now let's look at our whole process. Draw out what happens with [7, 2, 4, 1, 5, 3]:

```
Level 1 (Split):      [7, 2, 4, 1, 5, 3]
                      /                \
Level 2:        [7, 2, 4]          [1, 5, 3]
                /         \         /        \
Level 3:     [7]       [2, 4]    [1]      [5, 3]
                      /     \            /      \
Level 4:           [2]     [4]        [5]      [3]
```

Now when we merge back up, we get another tree, but upside down:

```
Level 4:           [2]     [4]        [5]      [3]
                      \     /            \      /
Level 3:            [2, 4]              [3, 5]
                         \                /
Level 2:              [2, 4, 7]    [1, 3, 5]
                              \    /
Level 1 (Merge):       [1, 2, 3, 4, 5, 7]
```

Look carefully at each level. How many total elements are we handling at each level?

- Level 4: [2], [4], [5], [3] -> 4 numbers
- Level 3: [2, 4], [3, 5] -> 4 numbers
- Level 2: [2, 4, 7], [1, 3, 5] -> 6 numbers
- Level 1: [1, 2, 3, 4, 5, 7] -> 6 numbers

Try this with your own example - do you notice something about the total numbers at each level?

[Space for thinking...]

The key insight: At each level, we're still working with all N numbers, just split up differently!

And at each level, we're doing a total of N steps of work (either splitting or merging).

So our total work is:

```
(N steps per level) × (number of levels)
```

But how many levels are there? Look back at our trees. What determines the number of levels?

[Space for thinking...]

Each split divides our groups in half. We keep splitting until we can't anymore (when we hit single elements). How many times can you divide N by 2 before you get to 1?

Let's try with N = 8:
8 → 4 → 2 → 1 (3 steps)
With N = 16:
16 → 8 → 4 → 2 → 1 (4 steps)

This special number (how many times you can divide by 2) has a name - it's the logarithm base 2! We write it as log₂(N).

## Putting It All Together

Let's summarize what we've discovered:

1. At each level of our trees:
   - We handle all N elements
   - Either splitting them up (in the first tree)
   - Or merging them together (in the second tree)

2. The number of levels:
   - Each split divides groups in half
   - We stop when we reach size 1
   - For N=16: 16 → 8 → 4 → 2 → 1 (4 levels)
   - For N=32: 32 → 16 → 8 → 4 → 2 → 1 (5 levels)
   - This is log₂(N) levels

3. Total work:
   - N work per level
   - log₂(N) levels
   - Two trees (splitting and merging)
   - So: 2 × N × log₂(N) total steps

This means our algorithm is O(N log N). Is this better than our original O(N²)?

Let's compare with some actual numbers:

```
N     |  N²   |  N log₂(N)
-------------------------
8     |  64   |  24
16    |  256  |  64
32    |  1024 |  160
64    |  4096 |  384
128   |  16384|  896
```

Try adding a few more rows yourself! What do you notice about how these numbers grow?

[Space for calculations...]

Remember our original problem? We wanted to do better than checking every element against every other element (N²). And we did! Instead of the work doubling four times when we double N (like in N²), it only doubles three times (like in N log N).

## The Final Algorithm

Here's our complete merge sort implementation:

```java
public class MergeSort {
    public static void sort(int[] arr) {
        // Base case: arrays of size 0 or 1 are sorted
        if (arr.length <= 1) return;
        
        // Split array in half
        int mid = arr.length / 2;
        int[] left = Arrays.copyOfRange(arr, 0, mid);
        int[] right = Arrays.copyOfRange(arr, mid, arr.length);
        
        // Recursively sort each half
        sort(left);
        sort(right);
        
        // Merge the sorted halves
        merge(arr, left, right);
    }
    
    private static void merge(int[] arr, int[] left, int[] right) {
        int leftPos = 0, rightPos = 0, arrPos = 0;
        
        // While we have elements in both halves,
        // take the smaller of the two
        while (leftPos < left.length && rightPos < right.length) {
            if (left[leftPos] <= right[rightPos]) {
                arr[arrPos++] = left[leftPos++];
            } else {
                arr[arrPos++] = right[rightPos++];
            }
        }
        
        // Copy remaining elements from left half
        while (leftPos < left.length) {
            arr[arrPos++] = left[leftPos++];
        }
        
        // Copy remaining elements from right half
        while (rightPos < right.length) {
            arr[arrPos++] = right[rightPos++];
        }
    }
}
```

What started as a simple idea - "what if we split the array in half?" - led us to discover one of computer science's most elegant and efficient sorting algorithms. By carefully thinking about how to combine sorted pieces and understanding the role of recursion, we created something better than our original approach.

Can you think of any ways to make it even better?

- What if we wanted to sort strings instead of numbers?
- What if we wanted to avoid creating new arrays for each split?
- What if we had so many numbers they didn't all fit in memory?

These questions lead to even more interesting algorithms... but that's a story for another day!
