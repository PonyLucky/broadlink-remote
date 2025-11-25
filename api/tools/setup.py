import broadlink

pwd = input("Enter the Broadlink password: ")
print(f"Your password is: `{pwd}`")
input("Press Enter to continue...")
broadlink.setup('Margot-4G', pwd, 3)
