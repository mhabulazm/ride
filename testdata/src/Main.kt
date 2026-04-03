package com.example

data class User(val name: String, val age: Int)

fun main() {
    val users = listOf(
        User("Alice", 30),
        User("Bob", 25)
    )

    for (user in users) {
        when {
            user.age > 28 -> println("${user.name} is senior")
            else -> println("${user.name} is junior")
        }
    }
}
