.text
.global main
main:
	push %rbp
	mov %rsp, %rbp
	mov $1, %rax
	mov %rbp, %rsp
	pop %rbp
	ret
